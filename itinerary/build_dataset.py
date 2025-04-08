#build_dataset.py
import requests
from datetime import datetime
from flight import Flight

def build_dataset(parsed_routes, serp_api_key):
    flights = []
    flight_no_counter = 0

    for i in range(len(parsed_routes) - 1):
        from_city, from_date = parsed_routes[i]
        to_city, _ = parsed_routes[i + 1]
        outbound_date = from_date.strftime("%Y-%m-%d")

        params = {
            "engine": "google_flights",
            "departure_id": from_city,
            "arrival_id": to_city,
            "outbound_date": outbound_date,
            "api_key": serp_api_key,
            "type": 2
        }

        print(f"Querying: {from_city} -> {to_city} on {outbound_date}")
        res = requests.get("https://serpapi.com/search", params=params)
        data = res.json()
        
        # Process both best_flights and other_flights
        all_flights = []
        if 'best_flights' in data:
            all_flights.extend(data['best_flights'])
        if 'other_flights' in data:
            all_flights.extend(data['other_flights'])
        
        if not all_flights:
            print(f"No flights found for {from_city} -> {to_city}")
            continue

        for flight in all_flights:
            try:
                flight_info = flight['flights'][0]
                dep_time_str = flight_info['departure_airport']['time']
                arr_time_str = flight_info['arrival_airport']['time']
                
                # Parse dates with the correct format then convert to integer timestamps
                dep_time_dt = datetime.strptime(dep_time_str, "%Y-%m-%d %H:%M")
                arr_time_dt = datetime.strptime(arr_time_str, "%Y-%m-%d %H:%M")
                
                # Convert to integer timestamps
                dep_time = int(dep_time_dt.timestamp())
                arr_time = int(arr_time_dt.timestamp())
                
                airline = flight_info['airline']
                price = flight.get('price', 9999999999)  # Use a default value if price is not available

                flights.append(Flight(i, i + 1, dep_time, arr_time, airline, price, flight_no_counter))
                flight_no_counter += 1
            except Exception as e:
                print(f"Skipping one flight: {e}")

    return flights  # Return outside the loop
