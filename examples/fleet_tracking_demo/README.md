# fleet_tracking_demo

An application demonstrating DDS instances.

This application tracks the positions of taxis that are in service.
When a taxi starts a trip, it becomes a tracking target, and the Publisher begins sending its position at regular intervals.
When a taxi ends its trip, the Publisher notifies the Subscriber.

This demo simulates taxi trip start and end events by generating and deleting vehicles with timers and random numbers.
Vehicle positions are updated every second, a vehicle is added every five seconds, and every eight seconds each vehicle has a 30% chance of being deleted.
When a vehicle is deleted, the Publisher explicitly calls `dispose` to notify the Subscriber.

This behavior is implemented using DDS instances.

An instance groups samples with the same key value. Instance lifecycles can be managed independently, and QoS policies such as History and Deadline are applied per instance.
To use instances, specify the key used to identify an instance with the `#[key]` attribute when defining the data exchanged between the applications. Multiple `#[key]` attributes can be used.

The data exchanged by this application contains a vehicle ID and coordinates, as shown below.
The `vehicle_id` field has the `#[key]` attribute, which is used to identify an instance.
```
#[derive(DdsData)]
struct VehiclePosition {
    #[key]
    vehicle_id: String,
    latitude: f64,
    longitude: f64,
}
```

Because instances are the unit to which QoS policies such as History and Deadline are applied, setting History to KeepLast with a depth of 1 allows the application to handle only the latest sample for each instance.

Because this application is intended to distribute and receive the latest position information with as little latency as possible, it uses BestEffort for Reliability QoS and KeepLast with a depth of 1 for History. The latter is the default setting and is therefore not specified explicitly.

Considering how to implement the same application without instances demonstrates the usefulness of instances.
Vehicle additions and position updates could be implemented by sending `VehiclePosition` samples as before.
However, because there would be no way to communicate that a vehicle had been deleted, a `cmd` member would need to be added to `VehiclePosition`. The operation type could be specified as `b'u'` for vehicle additions and position updates and `b'd'` for vehicle deletion.
```
struct VehiclePosition {
    vehicle_id: String,
    latitude: f64,
    longitude: f64,
    cmd: u8,
}
```
This would not solve everything.
QoS issues such as History and Deadline would remain. How could the application handle only the latest position for each vehicle or detect that a vehicle's position had not been updated for a certain period?
Without instances, this would require one topic per vehicle and dynamically creating a Topic and its corresponding DataWriter/DataReader.
This would not be particularly problematic on the Publisher side, but the Subscriber would need to determine which Topics had been created by the Publisher.
Since a Topic consists of a type and a name, and the type is always `VehiclePosition`, the Publisher would need to communicate each Topic name to the Subscriber. This could easily be achieved by creating a topic with a type such as the following:
```
struct TopicName {
    name: String,
}
```
However, with this approach, before information about a newly created vehicle could reach the Subscriber, the Publisher would need to send a `TopicName`, the Subscriber would need to create the Topic and DataReader, and Discovery would need to match them. This would introduce a significant delay between vehicle creation and the arrival of its data.

## Running the demo

Run the Publisher and Subscriber in separate terminals.

```sh
cargo run --example fleet_tracking_demo_sub
cargo run --example fleet_tracking_demo_pub
```

Both programs must use the same domain ID and topic name. When communicating over a network instead of localhost, configure the appropriate network interface address in both the Publisher and Subscriber.
