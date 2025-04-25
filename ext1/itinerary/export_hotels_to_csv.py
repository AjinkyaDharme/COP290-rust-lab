import csv

def export_hotels_to_csv(hotels, filename):
    with open(filename, mode='w', newline='') as file:
        writer = csv.writer(file)
        # Write a header for hotels
        writer.writerow([
            "City".ljust(20), 
            "Hotel Name".ljust(30), 
            "Address".ljust(40), 
            "Checkin".ljust(20), 
            "Checkout".ljust(20), 
            "Price".ljust(10)
        ])
        for hotel in hotels:
            writer.writerow([
                str(hotel.city).ljust(20)[:20],
                hotel.hotel_name.ljust(30)[:30],
                hotel.address.ljust(40)[:40],
                hotel.checkin.strftime("%Y-%m-%d").ljust(20)[:20],
                hotel.checkout.strftime("%Y-%m-%d").ljust(20)[:20],
                str(hotel.price).ljust(10)[:10]
            ])