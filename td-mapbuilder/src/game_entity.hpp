#pragma once
#include "SFML/Graphics/Color.hpp"
#include "SFML/System/Vector2.hpp"
#include <SFML/Graphics.hpp>
#include <cassert>

struct entityBundle
{
    sf::Vector2f position;
    bool is_selected;
    sf::CircleShape shape;
};


class gameEntity
{
    public:
    float radius;
    sf::Color fill_color;
    sf::Color outline_color;
    std::vector<entityBundle> entities;

    gameEntity(float radius, sf::Color fill_color, sf::Color outline_color) : radius(radius), fill_color(fill_color), outline_color(outline_color){}
    void addEntity(sf::Vector2f position);
    void reconstructEntityShapes();
    void deleteEntity(int index);
    void deleteSelectedEntities();
    void deselectAll();
    void moveSelected(sf::Vector2f movement);
};
