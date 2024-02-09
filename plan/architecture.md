# Application Design
The application will need to be designed in a way that allows the user to input their data and immediately receive a predicted value, while also protecting and managing user accounts. 

First, the user would have to register and login with the application through forms, after which they would be sent to a profile manager to complete their profile. The user login information would be stored in the database and the passwords would be hashed and salted using the bcrypt library. The profile manager would ask for the user's full name, address, city, state, and zipcode, using the location of the user for the fuel quote prediction.

After finalizing their account information, the user would then be sent to the fuel quote form in order to input the necessary criteria of location, gallons requested, and company profit margin. The application would then access the backend Pricing Module to predict the rate of fuel and return the suggested price per gallon as well as the total amount due.

This fuel rate will also be stored into the database for future reference by the user in a fuel quote history section. 

# Development Methodology

### Kanboard
Our team will be using the Kanban Development Methodology because we can visualize our workflow on the board easily by utilizing color-coded tags and the calendar view. There will be 5 tags that will be attached to each Kanboard card or task: backend is brown, database is green, documentation is gray, frontend is yellow, and testing is orange.

Our board will be divided into five different stages: Todo, Planning, Development, Testing, and Done.

Each teammate may see each other's tasks and progress on each step, which allows a fluid and transparent development process within the team. The tasks are marked by their corresponding tags and the name of a teammate. The cards also keep track of the creation, modification, and movement of the task as it moves across the five stages.

Some notable pros of Kanban are that it allows for continuous flow and allows changes can be made at any time. It is also quite flexible as it does not impose strict constraints or meetings. You can also add deadlines, notes, and options to upload files, links, and even documents to the cards. 

### Testing
For code testing, we will have three types: unit testing, integration testing, and end-to-end testing. 

### Coverage methodology
Code coverage is a measure of how much of your source code is tested. Since we are using Rust to write our API, we will use the Rust compiler's code coverage implementation called the LLVM instrumentation-based coverage via the `-C instrument-coverage` compiler flag. This generates coverage for all functions and the instrumented binary will be saved to a new and unique file each time. We can also create coverage reports via `llvm-profdata merge` which combines raw profiles and generates the reports for detailed coverage of our source code.

### Github
On Github, each teammate will create a branch specific to their tasks in order to make concurrent changes without messing up the flow of the project. Once the tasks are committed, each teammate makes a pull request to the main branch and it is reviewed before being accepted or denied. Back on the kanboard, once the pull request is approved, the task will be dropped into the Done column. 

# High-level desgin/architecture
