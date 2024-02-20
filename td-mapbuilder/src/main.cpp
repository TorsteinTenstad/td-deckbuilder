#include "SFML/Graphics/CircleShape.hpp"
#include "SFML/Graphics/Color.hpp"
#include "SFML/System/Vector2.hpp"
#include <SFML/Graphics.hpp>
#include <iostream>
#include <vector>
#include "SFML/Window/Keyboard.hpp"
#include "SFML/Window/Window.hpp"
#include "file_sys.cpp"
#include "git.cpp"
#include "game_entity.hpp"

struct Globals{
    sf::RenderWindow window;
    sf::Vector2f map_texture_size;
} globals;



sf::Vector2f vectorRescaler(sf::Vector2f pos, sf::Vector2f from_scale, sf::Vector2f to_scale)
{
    float x = pos.x / from_scale.x * to_scale.x;
    float y = pos.y / from_scale.y * to_scale.y;
    return {x, y};
}

class MouseEvent
{          
    public:
    sf::Mouse mouse;
    bool pressed = false;
    bool released_this_frame = false;
    bool pressed_this_frame = true;
    bool moved_while_pressed = false;
    sf::Vector2f position = sf::Vector2f(0,0);
    sf::Vector2f cursor_movement = sf::Vector2f(0,0);
    void update()
    {
        sf::Vector2f new_pos = vectorRescaler(sf::Vector2f(mouse.getPosition(globals.window)),sf::Vector2f(globals.window.getSize()), globals.map_texture_size);
        cursor_movement = new_pos - position;
        position = new_pos;
        released_this_frame = false;
        pressed_this_frame = false;
        moved_while_pressed = pressed && (cursor_movement != sf::Vector2f(0,0) || moved_while_pressed);
        if (mouse.isButtonPressed(mouse.Left))
        {
            if(!pressed){pressed_this_frame = true;}
            pressed = true;
        }
        else
        {
            if(pressed){released_this_frame = true;}
            pressed = false;
        }
    }
};

class KeyboardEvent{
    public:
    sf::Keyboard keyboard;
    sf::Keyboard::Key del = sf::Keyboard::Key::Backspace;
    sf::Keyboard::Key esc = sf::Keyboard::Key::Escape;
    bool save;
    bool saved;
    void update()
    {
        save = false;
        if (keyboard.isKeyPressed(sf::Keyboard::LControl) && keyboard.isKeyPressed(sf::Keyboard::S)){
            if(!saved){save = true;}
        saved = true;
        }
        else{
            saved = false;
        }
    }
};

bool isWithinBoundary(sf::Vector2f relative_pos, sf::Vector2f size)
{
    return relative_pos.x > 0 && relative_pos.y > 0 && relative_pos.x < size.x && relative_pos.y < size.y; 
}

bool intersectCircle(sf::Vector2f pos, sf::Vector2f origin, float radius)
{
    float x = pos.x - origin.x;
    float y = pos.y - origin.y;
    return x*x + y*y < radius*radius;
}

int mouseEntitiesIntersection(sf::Vector2f pos, std::vector<entityBundle> entities, float radius)
{

    for(int i = 0; i < entities.size(); i ++)
    {
        if (intersectCircle(pos, entities[i].position, radius)){return i;}
    }
    return -1;
}

bool any(const std::vector<bool> b)
{
    for (auto && i : b)
    {
        if (i){return true;}
    }
    return false;
}
bool anySelected(const std::vector<entityBundle> entities)
{
    for (auto entity: entities)
    {
        if(entity.is_selected){return true;}
    }
    return false;
}


int main() {
    std::string project_folder = "projects/";
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
        if (userInput == "break"){return -1;}

        else if (userInput == "new") {
            project_name = createNewProject(project_folder);
            if (project_name == ""){return -1;};
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
    
    std::string project_path = project_folder + project_name;
    if(!initializeGitRepository(project_path)){return -1;}
    std::string background_path = findFileInDirectory(project_path, "map", {"png", "jpeg"});

    gameEntity game_entity = gameEntity(25, sf::Color(0,0,139, 128), sf::Color(0,0, 200));
    if(findFileInDirectory(project_path, "entities", {"json"}) != "")
    {
        game_entity = loadEntitiesFromFile(project_path + "/entities.json");
    }
    MouseEvent mouse_event;
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
    int deselect_id = -1;

    // Main loop
    while (globals.window.isOpen()) {
        // Process events
        sf::Event event{};
        while (globals.window.pollEvent(event)) {
            if (event.type == sf::Event::Closed)
                globals.window.close();
        }

        mouse_event.update();
        keyboard_event.update();
        
        int intersect_id = mouseEntitiesIntersection(mouse_event.position, game_entity.entities, game_entity.radius);
        if (mouse_event.pressed_this_frame && intersect_id >= 0)
        {
            deselect_id = game_entity.entities[intersect_id].is_selected ? intersect_id : -1;
            game_entity.entities[intersect_id].is_selected = true;
        }
        if (mouse_event.released_this_frame && isWithinBoundary(mouse_event.position, globals.map_texture_size)) 
        {
            if (intersect_id < 0 && !mouse_event.moved_while_pressed)
            {
                if (anySelected(game_entity.entities)){game_entity.deselectAll();}
                else{game_entity.addEntity(mouse_event.position);}
            }
            else if (intersect_id >= 0 && !mouse_event.moved_while_pressed && deselect_id == intersect_id)
            {
                game_entity.entities[intersect_id].is_selected = false;
                deselect_id = -1;
            }
        }

        if (mouse_event.pressed)
        {
            game_entity.moveSelected(mouse_event.cursor_movement);
        }
        if (keyboard_event.keyboard.isKeyPressed(keyboard_event.del) && !mouse_event.pressed)
        {
            game_entity.deleteSelectedEntities();
        }
        if (keyboard_event.keyboard.isKeyPressed(keyboard_event.esc) && !mouse_event.pressed)
        {
            game_entity.deselectAll();
        }
        if (keyboard_event.save){saveEntitiesToFile(project_path + "/entities.json", game_entity);}

        // Clear the window
        globals.window.clear(sf::Color::White);
        globals.window.draw(map_sprite);
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
        
        // Display what was drawn
        globals.window.display();
    }

    return 0;
}