#include "SFML/System/Vector2.hpp"
#include <SFML/Graphics.hpp>
#include <iostream>
#include <filesystem>

namespace fs = std::filesystem;

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
int createNewProject(std::string project_folder) {

    std::string project_name;
    std::cout << "Please create a name for the project: \n";
    while(true){
        std::cin >> project_name;
        if (project_name == "break"){return -1;}
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
        if(user_input== "break"){return -1;}
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
    return 0;
}


int main() {
    std::string project_folder = "projects/";
    if (!fs::is_directory(project_folder))
    {
        fs::create_directory(project_folder);
    }
    
    listSubdirectories(project_folder);
    std::string userInput;
    std::cout << "Select a project or choose 'new' to create a new project: ";
    std::cin >> userInput;

    std::string project_path;

    if (userInput == "new") {
        int a = createNewProject(project_folder);
        if(a == -1){return 0;}
    }
    else{project_path = project_folder + userInput;}

    std::string background_path = findFileInDirectory(project_path, "map", {"png", "jpeg"});

    // Create a window with SFML
    sf::RenderWindow window(sf::VideoMode(800, 600), "Td Mapbuilder");

    sf::Texture map;
    map.loadFromFile(background_path);
    sf::Sprite map_sprite;
    map_sprite.setTexture(map);

    // Main loop
    while (window.isOpen()) {
        // Process events
        sf::Event event{};
        while (window.pollEvent(event)) {
            if (event.type == sf::Event::Closed)
                window.close();
        }

        // Clear the window
        window.clear(sf::Color::White);
        window.setView(sf::View(sf::Vector2f(map_sprite.getTexture()->getSize())/2.f, sf::Vector2f(map_sprite.getTexture()->getSize())));
        
        window.draw(map_sprite);
        
        // Display what was drawn
        window.display();
    }

    return 0;
}