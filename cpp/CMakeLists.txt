# this is the build file for project 
# it is autogenerated by the xmake build system.
# do not edit by hand.

# project
cmake_minimum_required(VERSION 3.15.0)
cmake_policy(SET CMP0091 NEW)
project(clean LANGUAGES CXX)


# target
set(CMAKE_C_COMPILER "gcc")
set(CMAKE_CXX_COMPILER "g++")
add_executable(shs "")
set_target_properties(shs PROPERTIES OUTPUT_NAME "shs")
set_target_properties(shs PROPERTIES RUNTIME_OUTPUT_DIRECTORY "${CMAKE_SOURCE_DIR}/build/windows/x64/release")
target_include_directories(shs PRIVATE
    includes
)
target_compile_options(shs PRIVATE
    $<$<COMPILE_LANGUAGE:C>:-m64>
    $<$<COMPILE_LANGUAGE:CXX>:-m64>
    $<$<COMPILE_LANGUAGE:C>:-DNDEBUG>
    $<$<COMPILE_LANGUAGE:CXX>:-DNDEBUG>
)
set_target_properties(shs PROPERTIES CXX_EXTENSIONS OFF)
target_compile_features(shs PRIVATE cxx_std_20)
if(MSVC)
    target_compile_options(shs PRIVATE $<$<CONFIG:Release>:-Ox -fp:fast>)
else()
    target_compile_options(shs PRIVATE -O3)
endif()
if(MSVC)
else()
    target_compile_options(shs PRIVATE -fvisibility=hidden)
endif()
if(MSVC)
    set_property(TARGET shs PROPERTY
        MSVC_RUNTIME_LIBRARY "MultiThreaded$<$<CONFIG:Debug>:Debug>")
endif()
target_link_options(shs PRIVATE
    -m64
)
target_sources(shs PRIVATE
    src/main.cpp
    src/option.cpp
)

