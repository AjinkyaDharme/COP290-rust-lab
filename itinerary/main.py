from datetime import datetime
from build_dataset import build_dataset
from planner import Planner
from export_to_csv import export_flights_to_csv
from copy import deepcopy

# Example input
parsed_routes = [("IXR", datetime(2025, 4, 9)), ("DEL", datetime(2025, 4, 10)), ("BOM", datetime(2025, 4, 11))]
serp_api_key = "64071de58da6c527780106d59abd1527be18007d8f5f8d331f494a930bb8d20d"

# Step 1: Build Dataset
flights = build_dataset(parsed_routes, serp_api_key)
planner = Planner(flights)

# Step 2: Generate all optimal routes
# Using indices instead of airport codes
start_city = 0  # Index of IXR in parsed_routes
end_city = 2    # Index of BOM in parsed_routes

# Convert datetimes to timestamps for the planner methods
t1 = int(datetime(2025, 4, 8).timestamp())
t2 = int(datetime(2025, 4, 12).timestamp())  # Extended to allow for all flights

route1 = planner.least_flights_earliest_route(start_city, end_city, t1, t2)
route2 = planner.cheapest_route(start_city, end_city, t1, t2)
route3 = planner.least_flights_cheapest_route(start_city, end_city, t1, t2)

# Function to convert timestamp integers back to datetime objects for CSV export
def convert_flights_for_export(route):
    converted_route = []
    for flight in route:
        # Create a copy of the flight
        converted_flight = deepcopy(flight)
        # Convert timestamps to datetime objects
        converted_flight.departure_time = datetime.fromtimestamp(flight.departure_time)
        converted_flight.arrival_time = datetime.fromtimestamp(flight.arrival_time)
        converted_flight.fare = flight.fare  # Assuming fare is already in the correct format
        converted_route.append(converted_flight)
    return converted_route

# Step 3: Export to CSV
export_flights_to_csv(convert_flights_for_export(route1), "least_flights_earliest.csv",parsed_routes)
export_flights_to_csv(convert_flights_for_export(route2), "cheapest_route.csv",parsed_routes)
export_flights_to_csv(convert_flights_for_export(route3), "least_flights_cheapest.csv",parsed_routes)

print("All routes exported successfully.")
