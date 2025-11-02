/**
 * Kea DHCP Hook for NNOE etcd Integration
 * 
 * This hook synchronizes DHCP lease information with etcd for centralized tracking.
 * Implements Kea hook API callouts: 
 *   IPv4: lease4_offer, lease4_renew, lease4_release
 *   IPv6: lease6_offer, lease6_renew, lease6_release
 *   Expiration: lease4_expire, lease6_expire
 */

#include <dhcpsrv/lease.h>
#include <hooks/hooks.h>
#include <log/message_initializer.h>
#include <curl/curl.h>
#include <json/json.h>
#include <string>
#include <sstream>
#include <iostream>
#include <vector>
#include <openssl/bio.h>
#include <openssl/evp.h>
#include <openssl/buffer.h>
#include <ctime>

using namespace isc::hooks;
using namespace isc::dhcp;
using namespace isc::log;

// Hook configuration
static std::string etcd_endpoints = "http://127.0.0.1:2379";
static std::string etcd_prefix = "/nnoe/dhcp/leases";
static uint32_t lease_ttl = 3600;

// CURL write callback for HTTP responses
static size_t WriteCallback(void *contents, size_t size, size_t nmemb, void *userp) {
    ((std::string*)userp)->append((char*)contents, size * nmemb);
    return size * nmemb;
}

// Base64 encode using OpenSSL
static std::string base64_encode(const std::string& input) {
    BIO *bio, *b64;
    BUF_MEM *bufferPtr;

    b64 = BIO_new(BIO_f_base64());
    bio = BIO_new(BIO_s_mem());
    bio = BIO_push(b64, bio);

    BIO_set_flags(bio, BIO_FLAGS_BASE64_NO_NL);
    BIO_write(bio, input.c_str(), static_cast<int>(input.length()));
    BIO_flush(bio);

    BIO_get_mem_ptr(bio, &bufferPtr);
    std::string encoded(bufferPtr->data, bufferPtr->length);

    BIO_free_all(bio);

    return encoded;
}

// Delete lease from etcd
static bool delete_lease_from_etcd(const std::string& ip_address) {
    CURL *curl;
    CURLcode res;
    std::string readBuffer;
    
    curl = curl_easy_init();
    if (!curl) {
        return false;
    }

    // Build etcd key
    std::string key = etcd_prefix + "/" + ip_address;
    
    // Base64 encode key
    std::string key_b64 = base64_encode(key);
    
    // Build etcd v3 API URL (DELETE)
    std::string url = etcd_endpoints + "/v3/kv/deleterange";
    
    Json::Value etcd_request;
    etcd_request["key"] = key_b64;
    
    Json::StreamWriterBuilder builder;
    std::string etcd_json = Json::writeString(builder, etcd_request);

    curl_easy_setopt(curl, CURLOPT_URL, url.c_str());
    curl_easy_setopt(curl, CURLOPT_POSTFIELDS, etcd_json.c_str());
    curl_easy_setopt(curl, CURLOPT_WRITEFUNCTION, WriteCallback);
    curl_easy_setopt(curl, CURLOPT_WRITEDATA, &readBuffer);
    
    struct curl_slist *headers = NULL;
    headers = curl_slist_append(headers, "Content-Type: application/json");
    curl_easy_setopt(curl, CURLOPT_HTTPHEADER, headers);

    res = curl_easy_perform(curl);
    
    // Check response
    long response_code;
    curl_easy_getinfo(curl, CURLINFO_RESPONSE_CODE, &response_code);
    
    curl_easy_cleanup(curl);
    curl_slist_free_all(headers);

    return (res == CURLE_OK && (response_code == 200 || response_code == 201));
}

// Send lease to etcd
static bool sync_lease_to_etcd(const Lease4Ptr& lease, const std::string& operation) {
    CURL *curl;
    CURLcode res;
    std::string readBuffer;
    
    curl = curl_easy_init();
    if (!curl) {
        return false;
    }

    // Build JSON payload
    Json::Value lease_data;
    lease_data["ip"] = lease->addr_.toText();
    lease_data["hwaddr"] = lease->hwaddr_->toText(false);
    lease_data["state"] = static_cast<int>(lease->state_);
    lease_data["cltt"] = static_cast<Json::Int64>(lease->cltt_);
    lease_data["valid_lft"] = static_cast<Json::Int64>(lease->valid_lft_);
    lease_data["operation"] = operation;
    lease_data["timestamp"] = static_cast<Json::Int64>(time(nullptr));
    
    // Calculate expiration timestamp for lease expiration handling
    std::time_t expires_at = lease->cltt_ + lease->valid_lft_;
    lease_data["expires_at"] = static_cast<Json::Int64>(expires_at);

    Json::StreamWriterBuilder builder;
    std::string json_str = Json::writeString(builder, lease_data);

    // Build etcd key
    std::string key = etcd_prefix + "/" + lease->addr_.toText();
    
    // Build etcd v3 API URL (PUT)
    std::string url = etcd_endpoints + "/v3/kv/put";
    
    // Base64 encode key and value (etcd v3 API requires base64 encoding)
    std::string key_b64 = base64_encode(key);
    std::string value_b64 = base64_encode(json_str);
    
    Json::Value etcd_request;
    etcd_request["key"] = key_b64;
    etcd_request["value"] = value_b64;
    
    std::string etcd_json = Json::writeString(builder, etcd_request);

    curl_easy_setopt(curl, CURLOPT_URL, url.c_str());
    curl_easy_setopt(curl, CURLOPT_POSTFIELDS, etcd_json.c_str());
    curl_easy_setopt(curl, CURLOPT_WRITEFUNCTION, WriteCallback);
    curl_easy_setopt(curl, CURLOPT_WRITEDATA, &readBuffer);
    
    struct curl_slist *headers = NULL;
    headers = curl_slist_append(headers, "Content-Type: application/json");
    curl_easy_setopt(curl, CURLOPT_HTTPHEADER, headers);

    res = curl_easy_perform(curl);
    
    // Check response code for better error handling
    long response_code = 0;
    if (res == CURLE_OK) {
        curl_easy_getinfo(curl, CURLINFO_RESPONSE_CODE, &response_code);
    }
    
    curl_easy_cleanup(curl);
    curl_slist_free_all(headers);

    if (res != CURLE_OK) {
        std::cerr << "Kea etcd hook: curl error: " << curl_easy_strerror(res) << std::endl;
        return false;
    }

    if (response_code != 200 && response_code != 201) {
        std::cerr << "Kea etcd hook: etcd API error, response code: " << response_code << std::endl;
        std::cerr << "Response: " << readBuffer << std::endl;
        return false;
    }

    return true;
}

// Hook library version
extern "C" int version() {
    return (KEA_HOOKS_VERSION);
}

// Hook library load
extern "C" int load(LibraryHandle& handle) {
    // Read configuration
    ConstElementPtr endpoints = handle.getParameter("etcd_endpoints");
    if (endpoints && endpoints->getType() == Element::string) {
        etcd_endpoints = endpoints->stringValue();
    }
    
    ConstElementPtr prefix = handle.getParameter("prefix");
    if (prefix && prefix->getType() == Element::string) {
        etcd_prefix = prefix->stringValue();
    }
    
    ConstElementPtr ttl = handle.getParameter("ttl");
    if (ttl && ttl->getType() == Element::integer) {
        lease_ttl = ttl->intValue();
    }

    // Initialize CURL
    curl_global_init(CURL_GLOBAL_DEFAULT);
    
    return 0;
}

// Hook library unload
extern "C" int unload() {
    curl_global_cleanup();
    return 0;
}

// lease4_offer callout
extern "C" int lease4_offer(CalloutHandle& handle) {
    try {
        Lease4Ptr lease;
        handle.getArgument("lease4", lease);
        
        if (lease) {
            sync_lease_to_etcd(lease, "offer");
        }
    } catch (const std::exception& e) {
        // Log error but don't fail the lease
        std::cerr << "Kea etcd hook error in lease4_offer: " << e.what() << std::endl;
    } catch (...) {
        std::cerr << "Kea etcd hook: Unknown error in lease4_offer" << std::endl;
    }
    
    return 0;
}

// lease4_renew callout
extern "C" int lease4_renew(CalloutHandle& handle) {
    try {
        Lease4Ptr lease;
        handle.getArgument("lease4", lease);
        
        if (lease) {
            sync_lease_to_etcd(lease, "renew");
        }
    } catch (const std::exception& e) {
        std::cerr << "Kea etcd hook error in lease4_renew: " << e.what() << std::endl;
    } catch (...) {
        std::cerr << "Kea etcd hook: Unknown error in lease4_renew" << std::endl;
    }
    
    return 0;
}

// lease4_release callout
extern "C" int lease4_release(CalloutHandle& handle) {
    try {
        Lease4Ptr lease;
        handle.getArgument("lease4", lease);
        
        if (lease) {
            sync_lease_to_etcd(lease, "release");
            // Delete lease from etcd on release
            delete_lease_from_etcd(lease->addr_.toText());
        }
    } catch (const std::exception& e) {
        std::cerr << "Kea etcd hook error in lease4_release: " << e.what() << std::endl;
    } catch (...) {
        std::cerr << "Kea etcd hook: Unknown error in lease4_release" << std::endl;
    }
    
    return 0;
}

// lease4_expire callout - handles expired IPv4 leases
extern "C" int lease4_expire(CalloutHandle& handle) {
    try {
        Lease4Ptr lease;
        handle.getArgument("lease4", lease);
        
        if (lease) {
            sync_lease_to_etcd(lease, "expire");
            // Delete expired lease from etcd
            delete_lease_from_etcd(lease->addr_.toText());
        }
    } catch (const std::exception& e) {
        std::cerr << "Kea etcd hook error in lease4_expire: " << e.what() << std::endl;
    } catch (...) {
        std::cerr << "Kea etcd hook: Unknown error in lease4_expire" << std::endl;
    }
    
    return 0;
}

// IPv6 lease sync function (similar to IPv4)
static bool sync_lease6_to_etcd(const Lease6Ptr& lease, const std::string& operation) {
    CURL *curl;
    CURLcode res;
    std::string readBuffer;
    
    curl = curl_easy_init();
    if (!curl) {
        return false;
    }

    // Build JSON payload for IPv6 lease
    Json::Value lease_data;
    lease_data["ip"] = lease->addr_.toText();
    lease_data["type"] = static_cast<int>(lease->type_); // IA_NA, IA_PD, etc.
    lease_data["iaid"] = static_cast<Json::UInt64>(lease->iaid_);
    lease_data["duid"] = lease->duid_ ? lease->duid_->toText() : "";
    lease_data["state"] = static_cast<int>(lease->state_);
    lease_data["cltt"] = static_cast<Json::Int64>(lease->cltt_);
    lease_data["valid_lft"] = static_cast<Json::Int64>(lease->valid_lft_);
    lease_data["preferred_lft"] = static_cast<Json::Int64>(lease->preferred_lft_);
    lease_data["operation"] = operation;
    lease_data["timestamp"] = static_cast<Json::Int64>(time(nullptr));
    
    // Calculate expiration timestamp for IPv6 lease expiration handling
    std::time_t expires_at = lease->cltt_ + lease->valid_lft_;
    lease_data["expires_at"] = static_cast<Json::Int64>(expires_at);

    Json::StreamWriterBuilder builder;
    std::string json_str = Json::writeString(builder, lease_data);

    // Build etcd key (IPv6 addresses use brackets in key for clarity)
    std::string key = etcd_prefix + "/" + lease->addr_.toText();
    
    // Build etcd v3 API URL (PUT)
    std::string url = etcd_endpoints + "/v3/kv/put";
    
    // Base64 encode key and value
    std::string key_b64 = base64_encode(key);
    std::string value_b64 = base64_encode(json_str);
    
    Json::Value etcd_request;
    etcd_request["key"] = key_b64;
    etcd_request["value"] = value_b64;
    
    std::string etcd_json = Json::writeString(builder, etcd_request);

    curl_easy_setopt(curl, CURLOPT_URL, url.c_str());
    curl_easy_setopt(curl, CURLOPT_POSTFIELDS, etcd_json.c_str());
    curl_easy_setopt(curl, CURLOPT_WRITEFUNCTION, WriteCallback);
    curl_easy_setopt(curl, CURLOPT_WRITEDATA, &readBuffer);
    
    struct curl_slist *headers = NULL;
    headers = curl_slist_append(headers, "Content-Type: application/json");
    curl_easy_setopt(curl, CURLOPT_HTTPHEADER, headers);

    res = curl_easy_perform(curl);
    
    long response_code = 0;
    if (res == CURLE_OK) {
        curl_easy_getinfo(curl, CURLINFO_RESPONSE_CODE, &response_code);
    }
    
    curl_easy_cleanup(curl);
    curl_slist_free_all(headers);

    if (res != CURLE_OK) {
        std::cerr << "Kea etcd hook (IPv6): curl error: " << curl_easy_strerror(res) << std::endl;
        return false;
    }

    if (response_code != 200 && response_code != 201) {
        std::cerr << "Kea etcd hook (IPv6): etcd API error, response code: " << response_code << std::endl;
        return false;
    }

    return true;
}

// Delete IPv6 lease from etcd
static bool delete_lease6_from_etcd(const std::string& ip_address) {
    CURL *curl;
    CURLcode res;
    std::string readBuffer;
    
    curl = curl_easy_init();
    if (!curl) {
        return false;
    }

    std::string key = etcd_prefix + "/" + ip_address;
    std::string key_b64 = base64_encode(key);
    std::string url = etcd_endpoints + "/v3/kv/deleterange";
    
    Json::Value etcd_request;
    etcd_request["key"] = key_b64;
    
    Json::StreamWriterBuilder builder;
    std::string etcd_json = Json::writeString(builder, etcd_request);

    curl_easy_setopt(curl, CURLOPT_URL, url.c_str());
    curl_easy_setopt(curl, CURLOPT_POSTFIELDS, etcd_json.c_str());
    curl_easy_setopt(curl, CURLOPT_WRITEFUNCTION, WriteCallback);
    curl_easy_setopt(curl, CURLOPT_WRITEDATA, &readBuffer);
    
    struct curl_slist *headers = NULL;
    headers = curl_slist_append(headers, "Content-Type: application/json");
    curl_easy_setopt(curl, CURLOPT_HTTPHEADER, headers);

    res = curl_easy_perform(curl);
    
    long response_code;
    curl_easy_getinfo(curl, CURLINFO_RESPONSE_CODE, &response_code);
    
    curl_easy_cleanup(curl);
    curl_slist_free_all(headers);

    return (res == CURLE_OK && (response_code == 200 || response_code == 201));
}

// lease6_offer callout - IPv6 lease offer
extern "C" int lease6_offer(CalloutHandle& handle) {
    try {
        Lease6Ptr lease;
        handle.getArgument("lease6", lease);
        
        if (lease) {
            sync_lease6_to_etcd(lease, "offer");
        }
    } catch (const std::exception& e) {
        std::cerr << "Kea etcd hook error in lease6_offer: " << e.what() << std::endl;
    } catch (...) {
        std::cerr << "Kea etcd hook: Unknown error in lease6_offer" << std::endl;
    }
    
    return 0;
}

// lease6_renew callout - IPv6 lease renewal
extern "C" int lease6_renew(CalloutHandle& handle) {
    try {
        Lease6Ptr lease;
        handle.getArgument("lease6", lease);
        
        if (lease) {
            sync_lease6_to_etcd(lease, "renew");
        }
    } catch (const std::exception& e) {
        std::cerr << "Kea etcd hook error in lease6_renew: " << e.what() << std::endl;
    } catch (...) {
        std::cerr << "Kea etcd hook: Unknown error in lease6_renew" << std::endl;
    }
    
    return 0;
}

// lease6_release callout - IPv6 lease release
extern "C" int lease6_release(CalloutHandle& handle) {
    try {
        Lease6Ptr lease;
        handle.getArgument("lease6", lease);
        
        if (lease) {
            sync_lease6_to_etcd(lease, "release");
            // Delete IPv6 lease from etcd on release
            delete_lease6_from_etcd(lease->addr_.toText());
        }
    } catch (const std::exception& e) {
        std::cerr << "Kea etcd hook error in lease6_release: " << e.what() << std::endl;
    } catch (...) {
        std::cerr << "Kea etcd hook: Unknown error in lease6_release" << std::endl;
    }
    
    return 0;
}

// lease6_expire callout - handles expired IPv6 leases
extern "C" int lease6_expire(CalloutHandle& handle) {
    try {
        Lease6Ptr lease;
        handle.getArgument("lease6", lease);
        
        if (lease) {
            sync_lease6_to_etcd(lease, "expire");
            // Delete expired IPv6 lease from etcd
            delete_lease6_from_etcd(lease->addr_.toText());
        }
    } catch (const std::exception& e) {
        std::cerr << "Kea etcd hook error in lease6_expire: " << e.what() << std::endl;
    } catch (...) {
        std::cerr << "Kea etcd hook: Unknown error in lease6_expire" << std::endl;
    }
    
    return 0;
}

