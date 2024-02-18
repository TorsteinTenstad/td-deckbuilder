#include "SFML/Graphics/CircleShape.hpp"
#include "SFML/Graphics/Color.hpp"
#include "SFML/System/Vector2.hpp"
#include <SFML/Graphics.hpp>
#include <cassert>
#include <iostream>
#include "SFML/Window/Window.hpp"
#include "file_sys.cpp"
#include "git.cpp"


struct Globals{
    sf::RenderWindow window;
} globals;


class gameEntity
{
    public:
    float radius;
    sf::Color fill_color;
    sf::Color outline_color;
    std::vector<sf::Vector2f> positions;
    std::vector<bool> are_selected;
    std::vector<sf::CircleShape> shapes;

    gameEntity(float radius, sf::Color fill_color, sf::Color outline_color) : radius(radius), fill_color(fill_color), outline_color(outline_color){}

    void addEntity(sf::Vector2f position)
    {
        positions.push_back(position);
        shapes.emplace_back(radius, 400);
        int end_ix = shapes.size() - 1;
        shapes[end_ix].setFillColor(fill_color);
        shapes[end_ix].setOutlineColor(outline_color);
        are_selected.push_back(false);
    }
    void deleteEntity(int index)
    {
        assert(positions.size() > index);
        positions.erase(positions.begin() + index);
        shapes.erase(shapes.begin() + index);
        are_selected.erase(are_selected.begin() + index);
    }
    void deselectAll()
    {
        for (auto && i : are_selected)
        {
            i = false;
        }
    }
};

class MouseEvent
{          
    public:
    sf::Mouse mouse;
    bool pressed;
    bool released_this_frame;
    sf::Vector2f position;
    void update()
    {
        position = sf::Vector2f(mouse.getPosition(globals.window));
        released_this_frame = false;
        if (mouse.isButtonPressed(mouse.Left))
        {
            pressed = true;
        }
        else
        {
            if(pressed){released_this_frame = true;}
            pressed = false;
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

int mouseEntitiesIntersection(sf::Vector2f pos, std::vector<sf::Vector2f> entity_pos, float radius)
{

    for(int i = 0; i < entity_pos.size(); i ++)
    {
        if (intersectCircle(pos, entity_pos[i], radius)){return i;}
    }
    return -1;
}

sf::Vector2f vectorRescaler(sf::Vector2f pos, sf::Vector2f from_scale, sf::Vector2f to_scale)
{
    float x = pos.x / from_scale.x * to_scale.x;
    float y = pos.y / from_scale.y * to_scale.y;
    return {x, y};
}

bool any(std::vector<bool> b)
{
    for (auto && i : b)
    {
        if (i){return true;}
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

    gameEntity entities = gameEntity(25, sf::Color(0,0,139, 128), sf::Color(0,0, 200));
    MouseEvent mouse_event;


    // Create a window with SFML
    globals.window.create(sf::VideoMode(800, 600), "Td Mapbuilder");

    sf::Texture map;
    map.loadFromFile(background_path);
    sf::Sprite map_sprite;
    map_sprite.setTexture(map);
    sf::Vector2f map_texture_size = sf::Vector2f(map.getSize());
    globals.window.setView(sf::View(map_texture_size / 2.f, map_texture_size));

    // Main loop
    while (globals.window.isOpen()) {
        // Process events
        sf::Event event{};
        while (globals.window.pollEvent(event)) {
            if (event.type == sf::Event::Closed)
                globals.window.close();
        }

        mouse_event.update();
        sf::Vector2f mouse_pos = vectorRescaler(sf::Vector2f(mouse_event.position), sf::Vector2f(globals.window.getSize()), map_texture_size);
        if (mouse_event.released_this_frame && isWithinBoundary(mouse_pos, map_texture_size)) 
        {   
            int select_id = mouseEntitiesIntersection(mouse_pos, entities.positions, entities.radius);
            if (any(entities.are_selected) && select_id < 0)
            {
                entities.deselectAll();
            }
            else if (select_id < 0)
            {
                entities.addEntity(mouse_pos);
            }
            else if(entities.are_selected[select_id])
            {
                entities.are_selected[select_id] = false;
            }
            else{
                entities.are_selected[select_id] = true;
            }
        }

        // Clear the window
        globals.window.clear(sf::Color::White);
        globals.window.draw(map_sprite);
        for (int i = 0; i < entities.positions.size(); i ++)
        {
            entities.shapes[i].setPosition(entities.positions[i]);
            entities.shapes[i].setOrigin(sf::Vector2f(entities.radius, entities.radius));
            if (entities.are_selected[i])
            {
                entities.shapes[i].setOutlineThickness(5.f);
            }
            else 
            {
                entities.shapes[i].setOutlineThickness(0.f);
            }
            globals.window.draw(entities.shapes[i]);
        }
        
        // Display what was drawn
        globals.window.display();
    }

    return 0;
}