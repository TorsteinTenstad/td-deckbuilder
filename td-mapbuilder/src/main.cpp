#include "SFML/Graphics/CircleShape.hpp"
#include "SFML/Graphics/Color.hpp"
#include "SFML/Graphics/Rect.hpp"
#include "SFML/Graphics/RectangleShape.hpp"
#include "SFML/System/Vector2.hpp"
#include <SFML/Graphics.hpp>
#include <iostream>
#include <vector>
#include <filesystem>
#include "SFML/Window/Window.hpp"
#include "file_sys.hpp"
#include "git.hpp"
#include "game_entity.hpp"
#include "cursor_and_keys.hpp"
#include "action.hpp"
#include "globals.hpp"

namespace fs = std::filesystem;

Globals globals;

gameEntity loadGameEntities(std::string project_path)
{
    if(findFileInDirectory(project_path, "entities", {"json"}) != "")
    {
        return loadEntitiesFromFile(project_path + "/entities.json");
    }
    else {
        return {25, sf::Color(0,0,139, 128), sf::Color(0,0, 200)};
    }
}

std::string initProject(std::string project_folder)
{
    if (!fs::is_directory(project_folder))
    {
        fs::create_directory(project_folder);
    }
    
    listSubdirectories(project_folder);

    std::string project_name;
    
    std::string userInput;
    std::cout << "Select a project or choose 'new' to create a new project: ";
    while(true){
        std::cin >> userInput;
        if (userInput == "break"){return "";}

        else if (userInput == "new") {
            project_name = createNewProject(project_folder);
            if (project_name == ""){return "";};
            break;
        }

        else if (findSubDirectory(project_folder, userInput) != ""){
            project_name = userInput;
            break;
        }
        else{
            std::cout << "Couldn't parse your command. Please open an existing project, make a new project with code \"new\" or write \"break\" to exit\n";
        }
    }
    
    return project_folder + project_name;
}

void drawEntities(gameEntity& game_entity)
{
    for (int i = 0; i < game_entity.entities.size(); i ++)
        {
            game_entity.entities[i].shape.setPosition(game_entity.entities[i].position);
            game_entity.entities[i].shape.setOrigin(sf::Vector2f(game_entity.radius, game_entity.radius));
            if (game_entity.entities[i].is_selected)
            {
                game_entity.entities[i].shape.setOutlineThickness(5.f);
            }
            else 
            {
                game_entity.entities[i].shape.setOutlineThickness(0.f);
            }
            globals.window.draw(game_entity.entities[i].shape);
        }
}

class UIElements
{
    public:
    sf::RectangleShape select_rectangle;

    UIElements()
    {
        select_rectangle.setFillColor(sf::Color(115,30,90, 128));
    }

};

void drawUI(UIElements& ui_elements, const MouseEvent& mouse_event, const ActionOptions& action_opts)
{
    if(mouse_event.moved_while_pressed && std::holds_alternative<select_drag>(action_opts.mode))
    {
        sf::Vector2f upper_left = sf::Vector2f(std::min(mouse_event.position.x, mouse_event.click_pos.x), std::min(mouse_event.position.y, mouse_event.click_pos.y));
        sf::Vector2f lower_right = sf::Vector2f(std::max(mouse_event.position.x, mouse_event.click_pos.x), std::max(mouse_event.position.y, mouse_event.click_pos.y));
        ui_elements.select_rectangle.setPosition(upper_left);
        ui_elements.select_rectangle.setSize(lower_right - upper_left);
        globals.window.draw(ui_elements.select_rectangle);
    }
}


int main() {
    std::string project_folder = "projects/";
    std::string project_path = initProject(project_folder);
    if (project_path == ""){return -1;}

    gitHandler git_handler = gitHandler(project_path);
    std::string background_path = findFileInDirectory(project_path, "map", {"png", "jpeg"});

    gameEntity game_entity = loadGameEntities(project_path);
    UIElements ui_elements;
    MouseEvent mouse_event;
    ActionOptions action_opts;
    ActionMode action;
    KeyboardEvent keyboard_event;


    // Create a window with SFML
    globals.window.create(sf::VideoMode(800, 600), "Td Mapbuilder");

    sf::Texture map;
    map.loadFromFile(background_path);
    sf::Sprite map_sprite;

    map_sprite.setTexture(map);
    globals.map_texture_size = sf::Vector2f(map.getSize());
    globals.window.setView(sf::View(globals.map_texture_size / 2.f, globals.map_texture_size));
    std::vector<bool> allow_deselect;

    // Main loop
    while (globals.window.isOpen()) {
        // Process events
        sf::Event event{};
        while (globals.window.pollEvent(event)) {
            if (event.type == sf::Event::Closed)
                globals.window.close();
        }
        if(!globals.window.hasFocus()){continue;}

        mouse_event.update();
        keyboard_event.update();
        action.compute(game_entity, keyboard_event, mouse_event, action_opts);

        if(action_opts.will_save){
            // std::cout << "Save" << "\n";
            saveEntitiesToFile(project_path + "/entities.json", game_entity);
            git_handler.stageAndCommit({"entities.json"});
            saveCommitIdToFile(project_path + "/oid.txt", git_handler.commit_ids.back());
        }
        if (action_opts.will_undo)
        {
            git_handler.Undo();
            game_entity = loadGameEntities(project_path);
        }
        if (action_opts.will_redo)
        {
            git_handler.Redo();
            game_entity = loadGameEntities(project_path);
        }
        if(action_opts.will_save || action_opts.will_undo || action_opts.will_redo){action_opts.has_unsaved_changes = false;}
        executeAction(game_entity, keyboard_event, mouse_event, action_opts);

        // Clear the window
        globals.window.clear(sf::Color::White);
        globals.window.draw(map_sprite);

        // Draw map entities and UI-layer
        drawUI(ui_elements, mouse_event, action_opts);
        drawEntities(game_entity);

        // Display what was drawn
        globals.window.display();
    }

    return 0;
}
