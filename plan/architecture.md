# Application Design

# Development Methodology

# High-level desgin/architecture
1. **Frontend Components**
   - **User Interface:** Frontend will consist several forms and tables containing information from the back-end database.
   - **Login/Register Pages:** These pages will allow the user to either log-in with their saved credentials or register if they are new users.
   - **Profile Management Page:** Users will be able to update their profile information after logging in.
   - **Fuel Quote Form:** This form will collect information such as client location, client history, gallons requested, and company profit margin.
   - **Fuel Quote History Page:** This page will display the history of fuel quotes for the current user.
2. **Backend Components**
   - **Authentication Service:** This service will handle user authentication and authorization, including login and registration functionality.
   - **Database:** Will store user data, including client profiles and fuel quote history.
   - **Fuel Quote Calculations:** Program responsible for calculationg fuel rates based on the provided criteria.
   - **Profile Managment:** Manages client profiles, including client registration, and updating existing users.
3. **Integration**
   - **APIs:** Backend services will expose APIs for communication with the frontend.
   - **Data Flow:** User inputs from the frontend will be sent to the backend, processed, and the results will be sent back to the frontend for display.
4. **Testing**
   - **Unit Testing:** Write unit testing for backend services to ensure individual componenents function correctly
   - **Integration Testing:** Test the interactions between frontend and backend services.
   - **End-to-End Testing:** Conduct end-to-end testing to ensure the application works as a whole from user interaction to backend processing.