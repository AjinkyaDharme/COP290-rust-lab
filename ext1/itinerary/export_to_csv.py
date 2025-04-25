# export_to_csv.py
import csv

def export_flights_to_csv(route, filename,parsed_routes):
    with open(filename, mode='w', newline='') as file:
        writer = csv.writer(file)
        # Write the header row
        writer.writerow([
            "Flight No".ljust(20), 
            "From".ljust(20), 
            "To".ljust(20), 
            "Departure".ljust(20), 
            "Arrival".ljust(20), 
            "Airline".ljust(20), 
            "Fare".ljust(20)
        ])
        
        # Write each flight's data
        for flight in route:
            writer.writerow([
            str(flight.flight_no).ljust(20)[:20],
            #str(flight.start_city).ljust(20)[:20],
            str(parsed_routes[flight.start_city][0]).ljust(20)[:20],
            #str(flight.end_city).ljust(20)[:20],
            str(parsed_routes[flight.end_city][0]).ljust(20)[:20],
            flight.departure_time.strftime("%Y-%m-%d %H:%M").ljust(20)[:20],
            flight.arrival_time.strftime("%Y-%m-%d %H:%M").ljust(20)[:20],
            flight.airline.ljust(20)[:20],
            str(flight.fare).ljust(20)[:20]
            ])
