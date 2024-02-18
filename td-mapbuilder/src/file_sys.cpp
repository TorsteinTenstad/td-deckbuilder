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
