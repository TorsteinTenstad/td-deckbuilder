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

    void addEntity(sf::Vector2f position)
    {
        entities.emplace_back(entityBundle{position, true, sf::CircleShape(radius)});
        entities.back().shape.setFillColor(fill_color);
        entities.back().shape.setOutlineColor(outline_color);
    }
    void reconstructEntityShapes()
    {
        for(entityBundle& entity: entities)
        {
            entity.shape.setRadius(radius);
            entity.shape.setFillColor(fill_color);
            entity.shape.setOutlineColor(outline_color);
        }
    }
    void deleteEntity(int index)
    {
        assert(entities.size() > index);
        entities.erase(entities.begin() + index);
    }
    void deleteSelectedEntities()
    {
        entities.erase(std::remove_if(entities.begin(), entities.end(), [](entityBundle entity){return entity.is_selected;}), entities.end());
    }
    void deselectAll()
    {
        for (auto& entity: entities)
        {
            entity.is_selected = false;
        }
    }
    void moveSelected(sf::Vector2f movement)
    {
        for(auto& entity: entities)
        {
            if (entity.is_selected){entity.position+= movement;}
        }
    }
};
