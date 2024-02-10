#include <SFML/Graphics.hpp>

int main() {
    // Create a window with SFML
    sf::RenderWindow window(sf::VideoMode(800, 600), "SFML Window");

    // Main loop
    while (window.isOpen()) {
        // Process events
        sf::Event event;
        while (window.pollEvent(event)) {
            if (event.type == sf::Event::Closed)
                window.close();
        }

        // Clear the window
        window.clear(sf::Color::White);

        // Draw your graphics here
        // Example:
        // sf::CircleShape shape(100.f);
        // shape.setFillColor(sf::Color::Green);
        // window.draw(shape);

        // Display what was drawn
        window.display();
    }

    return 0;
}
 