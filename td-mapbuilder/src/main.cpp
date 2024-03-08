#include "SFML/Graphics/CircleShape.hpp"
#include "SFML/Graphics/Color.hpp"
#include "SFML/System/Vector2.hpp"
#include <SFML/Graphics.hpp>
#include <iostream>
#include <optional>
#include <utility>
#include <vector>
#include "SFML/Window/Keyboard.hpp"
#include "SFML/Window/Window.hpp"
#include "file_sys.cpp"
#include "git.cpp"
#include "game_entity.hpp"
#include "vector.cpp"



struct Globals{
    sf::RenderWindow window;
    sf::Vector2f map_texture_size;
} globals;


class MouseEvent
{          
    public:
    sf::Mouse mouse;
    bool pressed = false;
    bool released_this_frame = false;
    bool pressed_this_frame = true;
    bool moved_while_pressed = false;
    sf::Vector2f position = {0,0};
    sf::Vector2f click_pos = {0,0};
    sf::Vector2f cursor_movement = {0,0};
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
            if(!pressed)
            {
                pressed_this_frame = true;
                click_pos = position;    
            }
            pressed = true;
        }
        else
        {
            if(pressed){released_this_frame = true;}
            pressed = false;
        }
    }
};

class oneFrameKey{
    private:
    bool pressed = false;
    std::vector<sf::Keyboard::Key> keys; 
    public:
    bool this_frame = false;
    oneFrameKey(std::vector<sf::Keyboard::Key> keys) : keys(std::move(keys)){};
    void Update(sf::Keyboard& keyboard)
    {
        this_frame = false;
        for(sf::Keyboard::Key& key: keys)
        {
            if(!keyboard.isKeyPressed(key))
            {
                pressed = false;
                return;
            }
        }
        if(!pressed){this_frame = true;}
        pressed = true;
    }
};

class KeyboardEvent{
    private:
    sf::Keyboard keyboard;
    public:
    sf::Keyboard::Key del = sf::Keyboard::Key::Backspace;
    sf::Keyboard::Key esc = sf::Keyboard::Key::Escape;
    sf::Keyboard::Key ctrl = sf::Keyboard::Key::LControl;
    sf::Keyboard::Key shift = sf::Keyboard::Key::LShift;
    bool shift_down = false;
    bool ctrl_down = false;
    bool esc_down = false;
    bool del_down = false;
    oneFrameKey save = {{ctrl, sf::Keyboard::Key::S}};
    oneFrameKey undo = {{ctrl, sf::Keyboard::Key::Z}};
    oneFrameKey redo = {{ctrl, sf::Keyboard::Key::Y}};
    void update()
    {
        save.Update(keyboard);
        undo.Update(keyboard);
        redo.Update(keyboard);
        ctrl_down = keyboard.isKeyPressed(ctrl);
        shift_down = keyboard.isKeyPressed(shift);
        esc_down = keyboard.isKeyPressed(esc);
        del_down = keyboard.isKeyPressed(del);
    }
};

int numSelected(const std::vector<entityBundle> entities)
{
    int num = 0;
    for (auto entity: entities)
    {
        if(entity.is_selected){num += 1;}
    }
    return num;
}

enum Action {select, move, add, del, none};

class ActionSystem
{
    public:
    Action action = none;
    bool mass_select = true;
    std::vector<bool> mass_selected;
    bool attempt_global_deselect = false;
    bool pressed_with_control = false;
    bool will_save = false;
    bool will_undo = false;
    bool will_redo = false;
    bool to_head = false;
    int select_id = -1;
    int intersect_id = -1;
    int num_selected = 0;

    // void update(gameEntity& game_entity, const KeyboardEvent& kb_event, const MouseEvent& m_event)
    // {
    //     setAction(game_entity, kb_event, m_event);
    //     act(game_entity, kb_event, m_event);
    // }

    void setAction(const gameEntity& game_entity, const KeyboardEvent& kb_event, const MouseEvent& m_event)
    {
        num_selected = numSelected(game_entity.entities);
        intersect_id = mouseEntitiesIntersection(m_event.position, game_entity.entities, game_entity.radius);
        attempt_global_deselect = false;
        will_save = false;
        will_undo = false;
        will_redo = false;
        
        // 1. If mouse left click and control is pressed, game will be in add mode for duration of press. 
        // Nothing will change until release, when a new entity will be added. Release will always cause commit.
        if(kb_event.ctrl_down && m_event.pressed_this_frame)
        {
            pressed_with_control = true;
        }
        if(pressed_with_control)
        {
            action = add;
            if(m_event.released_this_frame)
            {
                attempt_global_deselect = true;
                pressed_with_control = false;
                will_save = true;
            }
            return;
        }
         
        // 2. Implicit: control is not pressed. When pressed this frame, 
        // if and only if the mouse is hovering an unselected entity will this yield 
        // a definite action: selecting said entity. Will not always lead to commit.
        if(m_event.pressed_this_frame && intersect_id >= 0 && !game_entity.entities[intersect_id].is_selected)
        {
            if(action != select){will_save = true;}
            action = select;
            attempt_global_deselect = true;
            select_id = intersect_id;
            return;
        }

        // 3. Moving while pressed will always give a definite action. If something is selected, enter move mode, if not,
        // add. This clause will be true on the frame of release. Assumes that changing num_selected is impossible during pressed.
        // (TODO: make a minimal movement amount (>> 0) so this isn't activated spuriously)
        if(m_event.moved_while_pressed)
        {
            if(intersect_id < 0 || num_selected == 0 || kb_event.shift_down || mass_select)
            {
                action = select;
                if(!mass_select)
                {
                    for (auto entity: game_entity.entities)
                    {
                        mass_selected.push_back(false);
                    }
                }
                will_save = true;
                mass_select = true;
            }
            else
            {
                if(action != move){will_save = true;}
                action = move;
            }
            return;
        }

        // 3.5 Cleanup after 3. This might be prone to bugs, because 3 will also be true in the release frame. 
        if(mass_select)
        {
            mass_selected.clear();
            mass_select = false;
        }

        // 4. Implicit: Did not move while pressed, and is now released. Intersection will determine whether to add or toggle select,
        // and not pressing shift will deselect everything else.
        if(m_event.released_this_frame)
        {
            attempt_global_deselect = true;
            if(intersect_id >= 0 || num_selected > 0)
            {
                if(action != select){will_save = true;}
                action = select;
            }
            else 
            {
                action = add;
                will_save = true;
            }
            return;
        }

        // 5. Keyboard shortcuts. Nothing happens if mouse button is pressed (to avoid deleting or unselecting during other actions).
        // Apart from that, this should be pretty straight forward (?).
        if(!m_event.pressed)
        {
            if(kb_event.save.this_frame && to_head)
            {
                will_save = true;
            }
            else if(kb_event.undo.this_frame)
            {
                will_undo = true;
                if(to_head)
                {
                    will_save = true;
                    to_head = false;
                }
            }
            else if (kb_event.redo.this_frame) {
                will_redo = true;
            }
            else if(kb_event.del_down)
            {
                action = del;
                if(num_selected > 0){will_save = true;}
            }
            else if(kb_event.esc_down)
            {
                attempt_global_deselect = true;
                if(num_selected>0){will_save = true;}
                action = select;
            }
            return;
        }

        // 6. Either not sufficient input or no input at all, and the system will remain in it's previous action.
        return;
    }

    void act(gameEntity& game_entity, const KeyboardEvent& kb_event, const MouseEvent& m_event)
    {
        //std::cout << action << "\n";
        if(will_redo || will_undo || will_save){to_head = false;}
        if(attempt_global_deselect && (!kb_event.shift_down || kb_event.esc_down)){game_entity.deselectAll();}
        if (action == move)
        {
            if (m_event.pressed)
            {
                game_entity.moveSelected(m_event.cursor_movement);
                to_head = true;
            }
        }
        if (action == add)
        {
            if (m_event.released_this_frame)
            {
                game_entity.addEntity(m_event.position);
                to_head = true;
            }
        }
        if (action == del)
        {
            game_entity.deleteSelectedEntities();
            to_head = true;
        }
        if (action == select)
        {
            if(m_event.moved_while_pressed)
            {
                to_head = true;
                sf::Vector2f upper_left = sf::Vector2f(std::min(m_event.position.x, m_event.click_pos.x), std::min(m_event.position.y, m_event.click_pos.y));
                sf::Vector2f lower_right = sf::Vector2f(std::max(m_event.position.x, m_event.click_pos.x), std::max(m_event.position.y, m_event.click_pos.y));
                for(int i = 0; i < game_entity.entities.size(); i ++)
                {
                    entityBundle& entity = game_entity.entities[i];
                    bool intersect = intersectRectangle(entity.position, upper_left, lower_right);
                    if (intersect){mass_selected[i] = true;}
                    if (mass_selected[i]){entity.is_selected = intersect;}
                }
            }
            else
            {
                if((m_event.pressed_this_frame || m_event.released_this_frame) && intersect_id >= 0)
                {
                    game_entity.entities[intersect_id].is_selected = true;
                }
                if(m_event.released_this_frame && intersect_id >= 0 && intersect_id != select_id)
                {
                    game_entity.entities[intersect_id].is_selected = false;
                    to_head = true;
                }
            }
        }
        if(m_event.released_this_frame){select_id = -1;}
    }
};


bool any(const std::vector<bool> b)
{
    for (auto && i : b)
    {
        if (i){return true;}
    }
    return false;
}

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


int main() {
    std::string project_folder = "projects/";
    std::string project_path = initProject(project_folder);
    if (project_path == ""){return -1;}

    gitHandler git_handler = gitHandler(project_path);
    std::string background_path = findFileInDirectory(project_path, "map", {"png", "jpeg"});

    gameEntity game_entity = loadGameEntities(project_path);
    MouseEvent mouse_event;
    ActionSystem actions;
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
        actions.setAction(game_entity, keyboard_event, mouse_event);

        if(actions.will_save){
            std::cout << "Save" << "\n";
            saveEntitiesToFile(project_path + "/entities.json", game_entity);
            git_handler.stageAndCommit({"entities.json"});
            saveCommitIdToFile(project_path + "/oid.txt", git_handler.commit_ids.back());
        }
        if (actions.will_undo)
        {
            git_handler.Undo(actions.to_head);
            game_entity = loadGameEntities(project_path);
        }
        if (actions.will_redo)
        {
            git_handler.Redo();
            game_entity = loadGameEntities(project_path);
        }
        actions.act(game_entity, keyboard_event, mouse_event);

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