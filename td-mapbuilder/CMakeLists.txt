cmake_minimum_required(VERSION 3.11)

project(MapBuilder)

include(FetchContent)

# Use FetchContent to download SFML from GitHub
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

# Add your executable and link SFML
add_executable(mapbuilder src/main.cpp)
target_link_libraries(mapbuilder sfml-graphics)

# Copy SFML DLL files to the output directory
add_custom_command(TARGET mapbuilder POST_BUILD
    COMMAND ${CMAKE_COMMAND} -E copy_directory
        "$<TARGET_FILE_DIR:sfml-graphics>"
        $<TARGET_FILE_DIR:mapbuilder>
)
