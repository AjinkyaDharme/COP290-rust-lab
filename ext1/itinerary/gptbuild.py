import requests
from datetime import datetime
from flight import Flight

def build_dataset(parsed_routes, serp_api_key):
    flights = []
    flight_no_counter = 0

    for i in range(len(parsed_routes) - 1):
        from_city, from_date = parsed_routes[i]
        to_city, _ = parsed_routes[i + 1]
        outbound_date = from_date.strftime("%Y-%m-%d") #make this a string

        params = {
            "engine": "google_flights",
            "departure_id": from_city,
            "arrival_id": to_city,
            "outbound_date": outbound_date,
            "api_key": serp_api_key,
            "type":2
        }

        print(f"Querying: {from_city} -> {to_city} on {outbound_date}")
        res = requests.get("https://serpapi.com/search", params=params)
        data = res.json()
        #save the json locally for debugging
        with open('debug.json', 'w') as f:
            f.write(res.text)

        try:
            best_flights = data['best_flights']
        except KeyError:
            print(f"No best flights found for {from_city} -> {to_city}")
            continue

        for flight in best_flights:
            try:
                flight_info = flight['flights'][0]
                dep_time = datetime.fromisoformat(flight_info['departure_airport']['time'])
                arr_time = datetime.fromisoformat(flight_info['arrival_airport']['time'])
                airline = flight_info['airline']
                price = flight['price']

                flights.append(Flight(i, i + 1, dep_time, arr_time, airline, price, flight_no_counter))
                flight_no_counter += 1
            except Exception as e:
                print(f"Skipping one flight: {e}")

        return flights
