
/// Simulated vehicle configuration
#[derive(Clone, Copy, Debug)]
pub struct Vehicle {
    /// Vehicle's ID
    pub(crate) id: VehicleId,
    /// Vehicle's location represented by coordination
    coord: Point2d,
    /// Vehicle's width in m.
    width: f64,
    /// Vehicle's length in m.
    length: f64,
    /// Vehicle's height in m.
    height: f64,
    /// Vehicle's wheelbase in m.
    wheel_base: f64,
    /// The acceleration model
    // axle: AccelerationModel,
    /// Vhicle's velocity in m/s.
    vel: f64,
}


/// The attributes of a simulated vehicle.
#[derive(Clone, Copy)]
pub struct VehicleAttributes {
    /// The vehicle width in m.
    pub width: f64,
    /// The vehicle length in m.
    pub length: f64,
    /// Distance from vehicle's center to center of wheel axle.
    pub wheel_base: f64,
    /// The desired gap to vehicle ahead in seconds.
    pub headway: f64,
    /// The maximum acceleration of the vehicle, in m/s^2.
    pub max_acc: f64,
    /// The comfortable deceleartion of the vehicle, a negative number in m/s^2
    pub comf_dev: f64
}

impl Vehicle {

    /// Creates a new vehicle.
    pub(crate) fn new(id: VehicleId, attributes: &VehicleAttributes) -> Self {
        Self {
            id,
            
        }
    }

}