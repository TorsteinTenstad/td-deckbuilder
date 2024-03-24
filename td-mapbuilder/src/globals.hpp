#pragma once
#include <SFML/Graphics.hpp>

struct Globals {
    sf::RenderWindow window;
    sf::Vector2f map_texture_size;
};

extern Globals globals;