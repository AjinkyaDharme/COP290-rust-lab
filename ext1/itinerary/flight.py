#flight.py
class Flight:
    def __init__(self, start_city, end_city, departure_time, arrival_time, airline, fare, flight_no, booking_token):
        self.start_city = start_city
        self.end_city = end_city
        self.departure_time = departure_time
        self.arrival_time = arrival_time
        self.airline = airline
        self.fare = fare
        self.flight_no = flight_no
        self.booking_token = booking_token  # Placeholder for booking token, if needed