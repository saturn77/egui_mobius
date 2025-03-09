# Welcome 

`egui_mobius` is a framework for  creating modular egui_application.

![alt text](../../assets/egui_mobius_logo.png)
##  Landscape of Rust GUI 

When evaluating GUI frameworks in Rust, there are several key features to consider:
- **Ease of Use**: How easy is it to get started and build applications?
- **Performance**: How well does the framework perform, especially for complex and dynamic UIs?
- **Cross-Platform Support**: Does the framework support multiple operating systems?
- **Widget Availability**: Does the framework provide a rich set of widgets for building UIs?
- **Community and Ecosystem**: Is there a strong community and ecosystem around the framework?

Based on these criteria, `egui` has emerged as a top contender for desktop applications. It offers a well-designed architecture, compelling widgets, and a strong focus on performance and ease of use. The immediate mode nature of `egui` allows for highly responsive and interactive user interfaces, making it a powerful tool for building modern desktop applications.

## egui

Building scalable UI applications with `egui` can be challenging due to its immediate mode nature. Immediate mode GUI frameworks redraw the entire UI every frame, which can make managing complex state and optimizing performance more difficult. However, `egui` offers compelling widgets and a well-designed architecture that make it a powerful tool for creating responsive and interactive user interfaces. By leveraging `egui_mobius`, developers can overcome some of these challenges by using a modular approach and integrating threading and concurrency features.

### Separation of Concerns

One of the primary mechanisms for making a GUI modular is the separation of concerns between the backend and frontend. This approach allows developers to independently develop and maintain the logic and presentation layers of the application. However, `egui` at present does not inherently support this separation well, which can lead to tightly coupled code and difficulties in managing complex applications.

`egui_mobius` addresses this by providing a clear separation between the backend and frontend. The backend handles the application logic, state management, and processing of events, while the frontend is responsible for rendering the user interface and handling user interactions. This separation is akin to a Möbius strip, where the backend and frontend are two sides of a single surface, continuously interacting and updating each other.

By adopting this modular approach, `egui_mobius` enables developers to build scalable and maintainable applications, leveraging the strengths of both `egui` and the Mobius framework.





## Goals

The main goal of egui_mobius is **modularity facilitating design reuse and scalability.**

Additional goals 
- Create a framework that facilitates using threading and concurrency features
- Create consistent design patterns for egui applications
- Provide integrated tooling for scaffolding new projects
- Provide stateful widgets and components  


