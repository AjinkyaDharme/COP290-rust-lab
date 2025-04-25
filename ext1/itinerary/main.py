import sys
from datetime import datetime, timedelta
import re, csv
from build_dataset import build_dataset
from planner import Planner
from copy import deepcopy
from fetch_hotels import get_cheapest_hotel

def parse_itinerary(input_str):
    segments = input_str.split("->")
    parsed = []
    for seg in segments:
        match = re.match(r"(\w+)\((\d{2}/\d{2}/\d{4})\)", seg.strip())
        if match:
            city = match.group(1)
            date_str = match.group(2)
            date = datetime.strptime(date_str, "%d/%m/%Y")
            parsed.append((city, date))
    return parsed

def main():
    # If itinerary string provided as a command-line argument, use it.
    if len(sys.argv) > 1:
        itinerary_str = sys.argv[1]
    else:
        itinerary_str = input("Enter itinerary string (e.g., IXR(26/04/2025)->DEL(27/04/2025)->BOM(29/04/2025)): ")

    parsed_routes = parse_itinerary(itinerary_str)
    serp_api_key = "64071de58da6c527780106d59abd1527be18007d8f5f8d331f494a930bb8d20d"

    # Set t1 as the day before the first flight and t2 as the day after the last flight
    first_flight_date = parsed_routes[0][1]
    last_flight_date = parsed_routes[-1][1]
    t1 = int((first_flight_date - timedelta(days=1)).timestamp())
    t2 = int((last_flight_date + timedelta(days=1)).timestamp())

    # Step 1: Build Dataset and Planner
    flights = build_dataset(parsed_routes, serp_api_key)
    planner = Planner(flights)

    # Step 2: Generate optimal flight route (for example, using cheapest_route)
    start_city = 0  
    end_city = len(parsed_routes) - 1  
    route = planner.cheapest_route(start_city, end_city, t1, t2)

    # Helper: convert flight timestamps from integer to datetime objects.
    def convert_flights_for_export(route):
        converted = []
        for flight in route:
            f = deepcopy(flight)
            f.departure_time = datetime.fromtimestamp(flight.departure_time)
            f.arrival_time = datetime.fromtimestamp(flight.arrival_time)
            converted.append(f)
        return converted

    converted_route = convert_flights_for_export(route)

    # Step 3: Query cheapest hotel stays for each destination
    hotels = []
    for idx, (city, date) in enumerate(parsed_routes):
        checkin_ts = int(date.timestamp())
        if idx < len(parsed_routes) - 1:
            checkout_date = parsed_routes[idx + 1][1]
        else:
            checkout_date = date + timedelta(days=1)
        checkout_ts = int(checkout_date.timestamp())
        hotel = get_cheapest_hotel(city, checkin_ts, checkout_ts, serp_api_key)
        if hotel:
            hotels.append(hotel)

    # Step 4: Merge flight and hotel details into one CSV.
    def export_itinerary_to_csv(flights, hotels, output_filename):
        def format_row(row):
            return [str(item).ljust(50) for item in row]

        with open(output_filename, mode='w', newline='', encoding='utf-8') as csvfile:
            writer = csv.writer(csvfile)
            
            # Flight Itinerary Section Header
            writer.writerow(format_row(["Flight Itinerary"]))
            writer.writerow(format_row([
                 "Flight Number", "Departure Airport", "Departure Time", 
                 "Arrival Airport", "Arrival Time", "Fare", "Airline", "Booking Link"
            ]))
            base_booking_url = "https://serpapi.com/booking?token="  # adjust base URL as needed
            
            # Write one row per flight segment
            for flight in flights:
                booking_link = (base_booking_url + flight.booking_token) if hasattr(flight, "booking_token") and flight.booking_token else ""
                writer.writerow(format_row([
                    flight.flight_no,
                    parsed_routes[flight.start_city][0],
                    flight.departure_time.strftime("%Y-%m-%d %H:%M"),
                    parsed_routes[flight.end_city][0],
                    flight.arrival_time.strftime("%Y-%m-%d %H:%M"),
                    flight.fare,
                    flight.airline,
                    booking_link,
                ]))
            
            # Separation row
            writer.writerow(format_row([""]))
            # Hotel Itinerary Section Header
            writer.writerow(format_row(["Hotel Itinerary"]))
            writer.writerow(format_row([
                "City", "Hotel Name", "Checkin Date", "Checkout Date", "Hotel Price", "Address"
            ]))
            for hotel in hotels:
                writer.writerow(format_row([
                    hotel.city,
                    hotel.hotel_name,
                    hotel.checkin.strftime("%Y-%m-%d"),
                    hotel.checkout.strftime("%Y-%m-%d"),
                    hotel.price,
                    hotel.address
                ]))
        print(f"Combined itinerary exported to {output_filename}")

    export_itinerary_to_csv(converted_route, hotels, "final_itinerary.csv")
    print("All routes and hotel stays exported successfully.")

if __name__ == '__main__':
    main()
