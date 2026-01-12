cmake_minimum_required(VERSION 3.22.1)
project("egor_demo")

# Add the shared demo code (demos/android/app/src/main/cpp -> demos/shared)
set(SHARED_DIR "${CMAKE_CURRENT_SOURCE_DIR}/../../../../../shared")

# Include egor_mobile headers (demos/android/app/src/main/cpp -> egor_mobile/include)
set(EGOR_INCLUDE_DIR "${CMAKE_CURRENT_SOURCE_DIR}/../../../../../../include")

# Find the prebuilt egor_mobile library (cpp -> main -> jniLibs)
set(EGOR_LIB_DIR "${CMAKE_CURRENT_SOURCE_DIR}/../jniLibs/${ANDROID_ABI}")

add_library(egor_mobile SHARED IMPORTED)
set_target_properties(egor_mobile PROPERTIES
    IMPORTED_LOCATION "${EGOR_LIB_DIR}/libegor_mobile.so"
    IMPORTED_NO_SONAME TRUE
)

# Build the demo library
add_library(egor_demo SHARED
    native-lib.cpp
    ${SHARED_DIR}/bouncing_boxes.c
)

target_include_directories(egor_demo PRIVATE
    ${EGOR_INCLUDE_DIR}
    ${SHARED_DIR}
)

# Link libraries
find_library(log-lib log)
find_library(android-lib android)

target_link_libraries(egor_demo
    egor_mobile
    ${log-lib}
    ${android-lib}
)
