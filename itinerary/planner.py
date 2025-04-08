#planner.py
from flight import Flight
def comparison_function(a, b):
    if a[1] < b[1]:
        return True
    else:
        return False
    
def comparison_function2(a, b):
    if a[4] < b[4]:
        return True
    elif a[4]>b[4]:
        return False
    else:
        if a[1]<b[1]:
            return True
        else:
            return False
class Heap:
    '''
    Class to implement a heap with general comparison function
    '''
    def __init__(self, comparison_function, init_array):
        def comp1(a, b):
            if a < b:
                return True
            else:
                return False
        
        self.comparison_function = comparison_function 
        if comparison_function == None:
            self.comparison_function = comp1   
        self.heap = init_array if init_array else []
        
        if self.heap:
            for i in reversed(range(len(self.heap) // 2)):
                self._heapify_down(i)

    def _heapify_up(self, index):
        while index > 0:
            parent = (index - 1) // 2
            if self.comparison_function(self.heap[index],self.heap[parent]):
                self.heap[parent],self.heap[index]=self.heap[index], self.heap[parent]
                index = parent
            else:
                break

    def _heapify_down(self, index):
        while True:
            min_index = index
            left = 2 * index + 1
            right = 2 * index + 2

            if left < len(self.heap) and self.comparison_function(self.heap[left], self.heap[min_index]):
                min_index = left

            if right < len(self.heap) and self.comparison_function(self.heap[right], self.heap[min_index]):
                min_index = right

            if min_index != index:
                self.heap[index], self.heap[min_index] = self.heap[min_index], self.heap[index]
                index = min_index
            else:
                break


    def insert(self, value):
        if self.heap!=None:
            self.heap.append(value)
            self._heapify_up(len(self.heap) - 1)

    def extract(self):
        if not self.heap:
            return None
        
        if len(self.heap) == 1:
            return self.heap.pop()

        root = self.heap[0]
        self.heap[0] = self.heap.pop()
        self._heapify_down(0)
        return root

    def top(self):
        if self.heap:
            return self.heap[0]
        return None

    def __len__(self):
        return len(self.heap)
    
    def get_at_i(self,i):
        if self.heap!=None:
            return self.heap[i]

class Planner:
    def __init__(self, flights):
        """The Planner

        Args:
            flights (List[Flight]): A list of information of all the flights (objects of class Flight)
        """
        self.total_flights = len(flights)
        self.number_of_cities=0
        a=flights
        self.flights=[None for i in range(self.total_flights)]
        for i in range(self.total_flights):
            self.flights[a[i].flight_no]=a[i]
        for i in range(self.total_flights):
            self.number_of_cities = max(self.number_of_cities, flights[i].start_city, flights[i].end_city)
        self.number_of_cities+=1

        pass
    def least_flights_earliest_route(self, start_city, end_city, t1, t2):
        return self.least_flights_ealiest_route(start_city, end_city, t1, t2)
    def least_flights_ealiest_route(self, start_city, end_city, t1, t2):
        """
        Return List[Flight]: A route from start_city to end_city, which departs after t1 (>= t1) and
        arrives before t2 (<=) satisfying: 
        The route has the least number of flights, and within routes with same number of flights, 
        arrives the earliest
        """
        if start_city<0 or end_city<0 or start_city>=self.number_of_cities or end_city>=self.number_of_cities or start_city==end_city :
                    return []
        adj_list = [[] for i in range(self.number_of_cities)]
        for i in range(self.total_flights):
            if self.flights[i].departure_time>=t1 and self.flights[i].arrival_time<=t2:
                adj_list[self.flights[i].start_city].append((self.flights[i].end_city,self.flights[i]))
        queue=[]
        queue.append((start_city,0,None))
        min_time=[t2+100 for i in range(self.number_of_cities)]
        for_prev_flight=[None for i in range(self.total_flights)]
        min_time[start_city]=0
        stop=0
        min_time_flight=None
        while queue:
            n=len(queue)
            queue2=[]
            min_timing=t2+100
            for i in range(n):
                current=queue[i]
                current_state=current[0]
                current_time=current[1]
                current_flightno=current[2]
                if current_state==end_city:
                    stop=1
                    if current_time<min_timing:
                        min_timing=current_time
                        min_time_flight=current_flightno
                if stop!=1:
                    for i in adj_list[current_state]:
                        if i[1].departure_time>=current_time:
                            if i[1].arrival_time+20<min_time[i[0]]:
                                min_time[i[0]]=i[1].arrival_time+20
                                currentflightusednumber=i[1].flight_no
                                for_prev_flight[i[1].flight_no]=current_flightno
                                queue2.append((i[0],i[1].arrival_time+20,currentflightusednumber))
            if stop==1:
                result=[]
                currenti=min_time_flight
                result.append(self.flights[currenti])
                while for_prev_flight[currenti]!=None:
                    currenti=for_prev_flight[currenti]
                    result.append(self.flights[currenti])
                result.reverse()
                return result
                
            queue=queue2
        return []


    def cheapest_route(self, start_city, end_city, t1, t2):
        """
        Return List[Flight]: A route from start_city to end_city, which departs after t1 (>= t1) and
        arrives before t2 (<=) satisfying: 
        The route is a cheapest route
        """
        if start_city<0 or end_city<0 or start_city>=self.number_of_cities or end_city>=self.number_of_cities or start_city==end_city :
                    return []
        adj_list = [[] for i in range(self.number_of_cities)]
        for i in range(self.total_flights):
            if self.flights[i].departure_time>=t1 and self.flights[i].arrival_time<=t2:
                adj_list[self.flights[i].start_city].append((self.flights[i].end_city,self.flights[i]))
        
        initial_array=[]
        heap=Heap(comparison_function,initial_array)
        for_prev_flight=[None for i in range(self.total_flights)]
        visited_flights=[None for i in range(self.total_flights)]
        
        heap.insert((start_city,0,0,None))
        while heap:
            current=heap.extract()
            current_state=current[0]
            current_cost=current[1]
            current_time=current[2]
            current_flight_no=current[3]
            if current_state==end_city:
                result=[]
                currenti=current_flight_no
                result.append(self.flights[currenti])
                while for_prev_flight[currenti]!=None:
                    currenti=for_prev_flight[currenti]
                    result.append(self.flights[currenti])
                result.reverse()
                return result    
            for i in adj_list[current_state]:
                if current_time<=i[1].departure_time:
                    if visited_flights[i[1].flight_no]==None:
                        visited_flights[i[1].flight_no]=1
                        for_prev_flight[i[1].flight_no]=current_flight_no
                        heap.insert((i[0],current_cost+i[1].fare,i[1].arrival_time+20,i[1].flight_no))
        return []
        
    def least_flights_cheapest_route(self, start_city, end_city, t1, t2):
        """
        Return List[Flight]: A route from start_city to end_city, which departs after t1 (>= t1) and
        arrives before t2 (<=) satisfying: 
        The route has the least number of flights, and within routes with same number of flights, 
        is the cheapest
        """
        if start_city<0 or end_city<0 or start_city>=self.number_of_cities or end_city>=self.number_of_cities or start_city==end_city:
            return []
        adj_list = [[] for i in range(self.number_of_cities)]
        for i in range(self.total_flights):
            if self.flights[i].departure_time>=t1 and self.flights[i].arrival_time<=t2:
                adj_list[self.flights[i].start_city].append((self.flights[i].end_city,self.flights[i]))
        for_prev_flight=[None for i in range(self.total_flights)]
        visited_flights=[None for i in range(self.total_flights)]
        initial_array=[]
        heap=Heap(comparison_function2,initial_array)
        heap.insert((start_city,0,0,None,0))
        while heap:
            current=heap.extract()
            current_state=current[0]
            current_cost=current[1]
            current_time=current[2]
            current_flight_no=current[3]
            current_number_of_flights=current[4]
            if current_state==end_city:
                result=[]
                currenti=current_flight_no
                result.append(self.flights[currenti])
                while for_prev_flight[currenti]!=None:
                    currenti=for_prev_flight[currenti]
                    result.append(self.flights[currenti])
                result.reverse()
                return result
            for i in adj_list[current_state]:
                if current_time<=i[1].departure_time:
                    if visited_flights[i[1].flight_no]==None:
                        visited_flights[i[1].flight_no]=1
                        for_prev_flight[i[1].flight_no]=current_flight_no
                        heap.insert((i[0],current_cost+i[1].fare,i[1].arrival_time+20,i[1].flight_no,current_number_of_flights+1))
        return []
        