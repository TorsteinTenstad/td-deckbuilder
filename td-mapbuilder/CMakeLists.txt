cmake_minimum_required(VERSION 3.11)

project(MapBuilder)

# Set the C++ standard to C++17

include(FetchContent)

# Fetch SFML from GitHub
FetchContent_Declare(
    SFML
    GIT_REPOSITORY https://github.com/SFML/SFML.git
    GIT_TAG        2.5.1
)


FetchContent_GetProperties(SFML)
if(NOT SFML_POPULATED)
FetchContent_Populate(SFML)
add_subdirectory(${sfml_SOURCE_DIR} ${sfml_BINARY_DIR})
endif()

# Fetch Libgit2 from GitHub
FetchContent_Declare(
    Libgit2
    GIT_REPOSITORY https://github.com/libgit2/libgit2.git
    GIT_TAG        v1.2.0 # You can specify a specific tag or branch here
)

FetchContent_MakeAvailable(Libgit2)
include_directories(${libgit2_SOURCE_DIR}/include)


FetchContent_Declare(json URL https://github.com/nlohmann/json/releases/download/v3.11.3/json.tar.xz)
FetchContent_MakeAvailable(json)


# Add your executable and link libraries
add_executable(mapbuilder src/main.cpp)
target_link_libraries(mapbuilder sfml-graphics)
target_link_libraries(mapbuilder git2)
target_link_libraries(mapbuilder nlohmann_json::nlohmann_json)

target_compile_features(mapbuilder PRIVATE cxx_std_17)

# Copy SFML DLL files to the output directory
add_custom_command(TARGET mapbuilder POST_BUILD
    COMMAND ${CMAKE_COMMAND} -E copy_directory
        "$<TARGET_FILE_DIR:sfml-graphics>"
        $<TARGET_FILE_DIR:mapbuilder>
)
add_custom_command(TARGET mapbuilder POST_BUILD
    COMMAND ${CMAKE_COMMAND} -E copy
    "${CMAKE_BINARY_DIR}/_deps/libgit2-build/git2.dll"
    $<TARGET_FILE_DIR:mapbuilder>
)

enable_testing()
add_subdirectory(tests)