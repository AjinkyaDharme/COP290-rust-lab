import requests
import json
from datetime import datetime
from hotel import Hotel

def get_cheapest_hotel(city, checkin_timestamp, checkout_timestamp, serp_api_key):
    checkin = datetime.fromtimestamp(checkin_timestamp).strftime("%Y-%m-%d")
    checkout = datetime.fromtimestamp(checkout_timestamp).strftime("%Y-%m-%d")
    params = {
         "engine": "google_hotels",
         "q": city,
         "check_in_date": checkin,
         "check_out_date": checkout,
         "api_key": serp_api_key
    }
    print(f"Querying hotels for {city} from {checkin} to {checkout}")
    res = requests.get("https://serpapi.com/search", params=params)
    data = res.json()

    # Output the complete JSON response to a file for debugging purposes.
    with open("hotels_api_response.json", "w", encoding="utf-8") as f:
        json.dump(data, f, indent=4)
    
    # Use the "properties" key from the JSON response.
    hotels = data.get("properties", [])
    if not hotels:
         print(f"No hotels found for {city}")
         return None

    cheapest = None
    for h in hotels:
         try:
            # Extract price from "rate_per_night" field (using "extracted_lowest")
            price = h.get("rate_per_night", {}).get("extracted_lowest", 999999)
         except Exception as e:
            price = 999999
         if cheapest is None or price < cheapest.price:
              cheapest = Hotel(
                  city=city,
                  hotel_name=h.get("name", "Unknown"),
                  address=h.get("link", "Unknown"),
                  checkin=datetime.strptime(checkin, "%Y-%m-%d"),
                  checkout=datetime.strptime(checkout, "%Y-%m-%d"),
                  price=price
              )
    return cheapest