cmake_minimum_required(VERSION 3.10)
project(kea_etcd_hook)

set(CMAKE_CXX_STANDARD 17)
set(CMAKE_CXX_STANDARD_REQUIRED ON)

# Kea include directories (adjust paths as needed)
set(KEA_INCLUDE_DIRS
    /usr/include/kea
    /usr/local/include/kea
)

# etcd-cpp dependencies
find_package(PkgConfig REQUIRED)
pkg_check_modules(LIBCURL REQUIRED libcurl)
pkg_check_modules(JSONCPP REQUIRED jsoncpp)

# OpenSSL for base64 encoding
find_package(OpenSSL REQUIRED)

# Source files
set(SOURCES
    src/libdhcp_etcd.cpp
)

# Create shared library
add_library(dhcp_etcd SHARED ${SOURCES})

target_include_directories(dhcp_etcd PRIVATE
    ${KEA_INCLUDE_DIRS}
    ${LIBCURL_INCLUDE_DIRS}
    ${JSONCPP_INCLUDE_DIRS}
)

target_link_libraries(dhcp_etcd
    ${LIBCURL_LIBRARIES}
    ${JSONCPP_LIBRARIES}
    OpenSSL::SSL
    OpenSSL::Crypto
)

# Install to Kea hooks directory
install(TARGETS dhcp_etcd
    LIBRARY DESTINATION /usr/lib/kea/hooks
)

