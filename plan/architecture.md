# Application Design
The application will need to be designed in a way that allows the user to input their data and immediately receive a predicted value, while also protecting and managing user accounts. 

First, the user would have to register and login with the application through forms, after which they would be sent to a profile manager to complete their profile. The user login information would be stored in the database and the passwords would be hashed and salted using the bcrypt library. The profile manager would ask for the user's full name, address, city, state, and zipcode, using the location of the user for the fuel quote prediction.

After finalizing their account information, the user would then be sent to the fuel quote form in order to input the necessary criteria of location, gallons requested, and company profit margin. The application would then access the backend Pricing Module to predict the rate of fuel and return the suggested price per gallon as well as the total amount due.

This fuel rate will also be stored into the database for future reference by the user in a fuel quote history section. 

# Development Methodology

# High-level desgin/architecture
