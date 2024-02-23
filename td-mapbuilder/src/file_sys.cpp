#pragma once
#include <fstream>
#include <iostream>
#include <filesystem>
#include "SFML/Graphics/Color.hpp"
#include "SFML/System/Vector2.hpp"
#include "game_entity.hpp"
#include "json.hpp"
#include "nlohmann/adl_serializer.hpp"
#include "nlohmann/detail/iterators/iteration_proxy.hpp"
#include "nlohmann/json_fwd.hpp"
#include "git.cpp"

namespace fs = std::filesystem;
using json = nlohmann::json;

template<> struct nlohmann::adl_serializer<sf::Vector2f>
{
    static void to_json(json& j, const sf::Vector2f& vector)
    {
        j = json{{"x", vector.x}, {"y", vector.y}};
    }
    static void from_json(const json& j, sf::Vector2f& vector)
    {
        j.at("x").get_to(vector.x);
        j.at("y").get_to(vector.y);
    }
};

template<> struct nlohmann::adl_serializer<sf::Color>
{
    static void to_json(json& j, const sf::Color& color)
    {
        j = json{{"a", color.a}, {"b", color.b}, {"g",color.g}, {"r", color.r}};
    }
    static void from_json(const json& j, sf::Color& color)
    {
        j.at("a").get_to(color.a);
        j.at("b").get_to(color.b);
        j.at("g").get_to(color.g);
        j.at("r").get_to(color.r);
    }
};
template<> struct nlohmann::adl_serializer<entityBundle>
{
    static void to_json(json& j, const entityBundle& entity)
    {
        j = json{{"position", entity.position}, {"is_selected", entity.is_selected}};
    }
    static void from_json(const json& j, entityBundle& entity)
    {
        j.at("position").get_to(entity.position);
        j.at("is_selected").get_to(entity.is_selected);
    }
};

template<> struct nlohmann::adl_serializer<gameEntity>
{
    static void to_json(json& j, const gameEntity& game_entity)
    {
        j = json{{"radius", game_entity.radius}, {"fill_color", game_entity.fill_color}, {"outline_color", game_entity.outline_color}, {"entities", game_entity.entities}};
    }
    static void from_json(const json& j, gameEntity& game_entity)
    {
        j.at("radius").get_to(game_entity.radius);
        j.at("fill_color").get_to(game_entity.fill_color);
        j.at("outline_color").get_to(game_entity.outline_color);
        j.at("entities").get_to(game_entity.entities);
    }
};

// Function to list subdirectories in a folder
void listSubdirectories(const std::string& folderPath) {
    std::cout << "Subdirectories in " << folderPath << ":\n";
    for (const auto& entry : fs::directory_iterator(folderPath)) {
        if (fs::is_directory(entry)) {
            std::cout << entry.path().filename() << "\n";
        }
    }
}

// Function to find a file with a specific name and type in the project directory
std::string findFileInDirectory(const std::string& projectPath, const std::string& fileName, const std::vector<std::string>& file_types) {

    fs::path directoryPath(projectPath);

    // Iterate through the files in the directory
    for (const auto& entry : fs::directory_iterator(directoryPath)) {

        if (!entry.is_regular_file() || entry.path().filename().stem() != fileName) {
            continue;
            // Check if the file matches the specified name and type
        }
        for (std::string file_type: file_types)
        { 
            if(entry.path().extension() == "." + file_type) 
            { 
                // Return the path to the file
                return entry.path().string();
            }
        }
    }
    // Return an empty string if the file is not found
    return "";
}

std::string findSubDirectory(const std::string& dir, const std::string& subdir) {

    fs::path directoryPath(dir);

    // Iterate through the files in the directory
    for (const auto& entry : fs::directory_iterator(directoryPath)) {

        if (entry.is_directory() && entry.path().filename() == subdir) {
            return entry.path().string();
        }
    }
    // Return an empty string if the file is not found
    return "";
}

// Function to create a new project
std::string createNewProject(std::string project_folder) {

    std::string project_name;
    std::cout << "Please create a name for the project: \n";
    while(true){
        std::cin >> project_name;
        if (project_name == "break"){return "";}
        if (findSubDirectory(project_folder, project_name) != ""){
            std::cout << "This project already exists, please choose another name or remove existing project \n";
            continue;
        }
        break;
    }
    
    std::string project_path = project_folder + project_name;


    // Prompt user to select an image file
    std::cout << "Select an image file for the background (do not add file suffix):\n";
    std::string background_path;
    std::string user_input;
    while (true)
    {
        std::cin >> user_input;
        if(user_input== "break"){return "";}
        background_path = findFileInDirectory(".", user_input, {"png", "jpeg"});
        if (background_path == ""){
            std::cout << "Could not find the image file, please reenter \n";
            continue;
        }
        break;
    }

    // Create a new project directory
    fs::create_directory(project_path);

    // Copy the selected image file to the project directory/
    fs::copy_file(background_path, project_path + "/map.png");
    return project_name;
}


void saveEntitiesToFile(std::string filename, const gameEntity& game_entity)
{
    std::ofstream f;
    f.open(filename);

    json j = game_entity;
    f << j << std::endl;
}

bool saveEntitiesAndCommit(const std::string project_path, const std::string filename, const gameEntity& game_entity)
{
    saveEntitiesToFile(project_path +"/"+ filename, game_entity);
    bool commited = gitStageAndCommit(project_path, filename);
    std::cout<<commited<<"\n";
    return commited;
}

gameEntity loadEntitiesFromFile(std::string filename)
{
    std::ifstream f;
    f.open(filename);

    json j;
    f >> j;
    gameEntity game_entity {j["radius"].template get<float>(),
                j["fill_color"].template get<sf::Color>(),
                j["outline_color"].template get<sf::Color>()
    };
    game_entity.entities = j["entities"].template get<std::vector<entityBundle>>();
    game_entity.reconstructEntityShapes();
    return game_entity;
}
