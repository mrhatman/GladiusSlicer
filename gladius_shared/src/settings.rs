#![deny(missing_docs)]

use crate::error::SlicerErrors;
use crate::types::{MoveType, PartialInfillTypes, SolidInfillTypes};
use crate::warning::SlicerWarnings;
use serde::{Deserialize, Serialize};

macro_rules! setting_less_than_or_equal_to_zero {
    ($settings:ident,$setting:ident) => {{
        if $settings.$setting as f64 <= 0.0 {
            return SettingsValidationResult::Error(SlicerErrors::SettingLessThanOrEqualToZero {
                setting: stringify!($setting).to_string(),
                value: $settings.$setting as f64,
            });
        }
    }};
}

macro_rules! option_setting_less_than_or_equal_to_zero {
    ($settings:ident,$setting:ident) => {{
        if let Some(temp) = $settings.$setting {
            if (temp as f64) <= 0.0 {
                return SettingsValidationResult::Error(
                    SlicerErrors::SettingLessThanOrEqualToZero {
                        setting: stringify!($setting).to_string(),
                        value: temp as f64,
                    },
                );
            }
        }
    }};
}

macro_rules! setting_less_than_zero {
    ($settings:ident,$setting:ident) => {{
        if ($settings.$setting as f64) < 0.0 {
            return SettingsValidationResult::Error(SlicerErrors::SettingLessThanZero {
                setting: stringify!($setting).to_string(),
                value: $settings.$setting as f64,
            });
        }
    }};
}

macro_rules! option_setting_less_than_zero {
    ($settings:ident,$setting:ident) => {{
        if let Some(temp) = $settings.$setting {
            if (temp as f64) < 0.0 {
                return SettingsValidationResult::Error(
                    SlicerErrors::SettingLessThanOrEqualToZero {
                        setting: stringify!($setting).to_string(),
                        value: temp as f64,
                    },
                );
            }
        }
    }};
}

///A complete settings file for the entire slicer.
#[derive(Serialize, Deserialize, Debug)]
pub struct Settings {
    ///The height of the layers
    pub layer_height: f64,

    ///The extrusion width of the layers
    pub extrusion_width: MovementParameter,

    ///The filament Settings
    pub filament: FilamentSettings,

    ///The fan settings
    pub fan: FanSettings,

    ///The skirt settings, if None no skirt will be generated
    pub skirt: Option<SkirtSettings>,

    ///The support settings, if None no support will be generated
    pub support: Option<SupportSettings>,

    ///Diameter of the nozzle in mm
    pub nozzle_diameter: f64,

    ///length to retract in mm
    pub retract_length: f64,

    ///Distance to lift the z axis during a retract
    pub retract_lift_z: f64,

    ///The velocity of retracts
    pub retract_speed: f64,

    ///Retraction Wipe
    pub retraction_wipe: Option<RetractionWipeSettings>,

    ///The speeds used for movement
    pub speed: MovementParameter,

    ///The acceleration for movement
    pub acceleration: MovementParameter,

    ///The percentage of infill to use for partial infill
    pub infill_percentage: f64,

    ///Controls the order of perimeters
    pub inner_perimeters_first: bool,

    ///Number of perimeters to use if possible
    pub number_of_perimeters: usize,

    ///Number of solid top layers for infill
    pub top_layers: usize,

    ///Number of solid bottom layers before infill
    pub bottom_layers: usize,

    ///Size of the printer in x dimension in mm
    pub print_x: f64,

    ///Size of the printer in y dimension in mm
    pub print_y: f64,

    ///Size of the printer in z dimension in mm
    pub print_z: f64,

    ///Width of the brim, if None no brim will be generated
    pub brim_width: Option<f64>,

    ///Inset the layer by the provided amount, if None on inset will be performed
    pub layer_shrink_amount: Option<f64>,

    ///The minimum travel distance required to perform a retraction
    pub minimum_retract_distance: f64,

    ///Overlap between infill and interior perimeters
    pub infill_perimeter_overlap_percentage: f64,

    ///Solid Infill type
    pub solid_infill_type: SolidInfillTypes,

    ///Partial Infill type
    pub partial_infill_type: PartialInfillTypes,

    ///The instructions to prepend to the exported instructions
    pub starting_instructions: String,

    ///The instructions to append to the end of the exported instructions
    pub ending_instructions: String,

    /// The instructions to append before layer changes
    pub before_layer_change_instructions: String,

    /// The instructions to append after layer changes
    pub after_layer_change_instructions: String,

    /// The instructions to append between object changes
    pub object_change_instructions: String,

    ///Maximum Acceleration in x dimension
    pub max_acceleration_x: f64,
    ///Maximum Acceleration in y dimension
    pub max_acceleration_y: f64,
    ///Maximum Acceleration in z dimension
    pub max_acceleration_z: f64,
    ///Maximum Acceleration in e dimension
    pub max_acceleration_e: f64,

    ///Maximum Acceleration while extruding
    pub max_acceleration_extruding: f64,
    ///Maximum Acceleration while traveling
    pub max_acceleration_travel: f64,
    ///Maximum Acceleration while retracting
    pub max_acceleration_retracting: f64,

    ///Maximum Jerk in x dimension
    pub max_jerk_x: f64,
    ///Maximum Jerk in y dimension
    pub max_jerk_y: f64,
    ///Maximum Jerk in z dimension
    pub max_jerk_z: f64,
    ///Maximum Jerk in e dimension
    pub max_jerk_e: f64,

    ///Minimum feedrate for extrusion moves
    pub minimum_feedrate_print: f64,
    ///Minimum feedrate for travel moves
    pub minimum_feedrate_travel: f64,
    ///Maximum feedrate for x dimension
    pub maximum_feedrate_x: f64,
    ///Maximum feedrate for y dimension
    pub maximum_feedrate_y: f64,
    ///Maximum feedrate for z dimension
    pub maximum_feedrate_z: f64,
    ///Maximum feedrate for e dimension
    pub maximum_feedrate_e: f64,

    ///Settings for specific layers
    pub layer_settings: Vec<(LayerRange, PartialLayerSettings)>,
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            layer_height: 0.15,
            number_of_perimeters: 3,
            top_layers: 3,
            bottom_layers: 3,
            extrusion_width: MovementParameter {
                interior_inner_perimeter: 0.4,
                interior_surface_perimeter: 0.4,
                exterior_inner_perimeter: 0.4,
                solid_top_infill: 0.4,
                solid_infill: 0.4,
                infill: 0.4,
                travel: 0.4,
                bridge: 0.4,
                support: 0.4,
                exterior_surface_perimeter: 0.4,
            },
            filament: FilamentSettings::default(),
            fan: FanSettings::default(),
            skirt: None,
            nozzle_diameter: 0.4,
            retract_length: 0.8,
            retract_lift_z: 0.6,
            retract_speed: 35.0,

            support: None,

            speed: MovementParameter {
                interior_inner_perimeter: 40.0,
                interior_surface_perimeter: 40.0,
                exterior_inner_perimeter: 40.0,
                solid_top_infill: 200.0,
                solid_infill: 200.0,
                infill: 200.0,
                travel: 180.0,
                bridge: 30.0,
                support: 50.0,
                exterior_surface_perimeter: 40.0,
            },
            acceleration: MovementParameter {
                interior_inner_perimeter: 900.0,
                interior_surface_perimeter: 900.0,
                exterior_inner_perimeter: 800.0,
                solid_top_infill: 1000.0,
                solid_infill: 1000.0,
                infill: 1000.0,
                travel: 1000.0,
                bridge: 1000.0,
                support: 1000.0,
                exterior_surface_perimeter: 800.0,
            },

            infill_percentage: 0.2,

            print_x: 210.0,
            print_y: 210.0,
            print_z: 210.0,
            inner_perimeters_first: true,
            minimum_retract_distance: 1.0,
            infill_perimeter_overlap_percentage: 0.25,
            solid_infill_type: SolidInfillTypes::Rectilinear,
            partial_infill_type: PartialInfillTypes::Linear,
            starting_instructions: "G90 ; use absolute coordinates \n\
                                M83 ; extruder relative mode\n\
                                M106 S255 ; FANNNNN\n\
                                M104 S[First Layer Extruder Temp] ; set extruder temp\n\
                                M140 S[First Layer Bed Temp] ; set bed temp\n\
                                M190 S[First Layer Bed Temp]; wait for bed temp\n\
                                M109 S[First Layer Extruder Temp] ; wait for extruder temp\n\
                                G28 W ; home all without mesh bed level\n\
                                G80 ; mesh bed leveling\n\
                                G1 Y-3.0 F1000.0 ; go outside print area\n\
                                G92 E0.0\n\
                                G1 X60.0 E9.0 F1000.0 ; intro line\n\
                                G1 X100.0 E12.5 F1000.0 ; intro line\n\
                                G92 E0.0;\n"
                .to_string(),
            ending_instructions: "G4 ; wait\n\
                                M221 S100 \n\
                                M104 S0 ; turn off temperature \n\
                                M140 S0 ; turn off heatbed \n\
                                G1 X0 F3000 ; home X axis \n\
                                M84 ; disable motors\n\
                                M107 ; disable fan\n"
                .to_string(),
            before_layer_change_instructions: "".to_string(),
            after_layer_change_instructions: "".to_string(),
            object_change_instructions: "".to_string(),
            max_acceleration_x: 1000.0,
            max_acceleration_y: 1000.0,
            max_acceleration_z: 1000.0,
            max_acceleration_e: 5000.0,
            max_acceleration_extruding: 1250.0,
            max_acceleration_travel: 1250.0,
            max_acceleration_retracting: 1250.0,
            max_jerk_x: 8.0,
            max_jerk_y: 8.0,
            max_jerk_z: 0.4,
            brim_width: None,
            layer_settings: vec![(
                LayerRange::SingleLayer(0),
                PartialLayerSettings {
                    extrusion_width: None,
                    speed: Some(MovementParameter {
                        interior_inner_perimeter: 20.0,
                        interior_surface_perimeter: 20.0,
                        exterior_inner_perimeter: 20.0,
                        solid_top_infill: 20.0,
                        solid_infill: 20.0,
                        infill: 20.0,
                        travel: 5.0,
                        bridge: 20.0,
                        support: 20.0,
                        exterior_surface_perimeter: 20.0,
                    }),
                    layer_height: Some(0.3),
                    bed_temp: Some(60.0),
                    extruder_temp: Some(210.0),
                    ..Default::default()
                },
            )],
            layer_shrink_amount: None,
            max_jerk_e: 1.5,
            minimum_feedrate_print: 0.0,
            minimum_feedrate_travel: 0.0,
            maximum_feedrate_x: 200.0,
            maximum_feedrate_y: 200.0,
            maximum_feedrate_z: 12.0,
            maximum_feedrate_e: 120.0,
            retraction_wipe: None,
        }
    }
}

impl Settings {
    ///Get the layer settings for a specific layer index and height
    pub fn get_layer_settings(&self, layer: usize, height: f64) -> LayerSettings {
        let changes = self
            .layer_settings
            .iter()
            .filter(|(layer_range, _)| match layer_range {
                LayerRange::LayerCountRange { end, start } => *start <= layer && layer <= *end,
                LayerRange::HeightRange { end, start } => *start <= height && height <= *end,
                LayerRange::SingleLayer(filter_layer) => *filter_layer == layer,
            })
            .map(|(_lr, pls)| pls)
            .fold(PartialLayerSettings::default(), |a, b| a.combine(b));

        LayerSettings {
            layer_height: changes.layer_height.unwrap_or(self.layer_height),
            layer_shrink_amount: changes.layer_shrink_amount.or(self.layer_shrink_amount),
            speed: changes.speed.unwrap_or_else(|| self.speed.clone()),
            acceleration: changes
                .acceleration
                .unwrap_or_else(|| self.acceleration.clone()),
            extrusion_width: changes
                .extrusion_width
                .unwrap_or_else(|| self.extrusion_width.clone()),
            solid_infill_type: changes.solid_infill_type.unwrap_or(self.solid_infill_type),
            partial_infill_type: changes
                .partial_infill_type
                .unwrap_or(self.partial_infill_type),
            infill_percentage: changes.infill_percentage.unwrap_or(self.infill_percentage),
            infill_perimeter_overlap_percentage: changes
                .infill_perimeter_overlap_percentage
                .unwrap_or(self.infill_perimeter_overlap_percentage),
            inner_perimeters_first: changes
                .inner_perimeters_first
                .unwrap_or(self.inner_perimeters_first),
            bed_temp: changes.bed_temp.unwrap_or(self.filament.bed_temp),
            extruder_temp: changes.extruder_temp.unwrap_or(self.filament.extruder_temp),
            retraction_wipe: changes
                .retraction_wipe
                .or_else(|| self.retraction_wipe.clone()),
            retraction_length: changes.retraction_length.unwrap_or(self.retract_length),
        }
    }

    ///Validate settings and return any warnings and errors
    pub fn validate_settings(&self) -> SettingsValidationResult {
        setting_less_than_or_equal_to_zero!(self, print_x);
        setting_less_than_or_equal_to_zero!(self, print_y);
        setting_less_than_or_equal_to_zero!(self, print_z);
        setting_less_than_or_equal_to_zero!(self, nozzle_diameter);
        setting_less_than_or_equal_to_zero!(self, layer_height);
        setting_less_than_or_equal_to_zero!(self, retract_speed);
        setting_less_than_or_equal_to_zero!(self, max_acceleration_x);
        setting_less_than_or_equal_to_zero!(self, max_acceleration_y);
        setting_less_than_or_equal_to_zero!(self, max_acceleration_z);
        setting_less_than_or_equal_to_zero!(self, max_acceleration_e);
        setting_less_than_or_equal_to_zero!(self, max_jerk_x);
        setting_less_than_or_equal_to_zero!(self, max_jerk_y);
        setting_less_than_or_equal_to_zero!(self, max_jerk_z);
        setting_less_than_or_equal_to_zero!(self, max_jerk_e);
        setting_less_than_or_equal_to_zero!(self, max_acceleration_extruding);
        setting_less_than_or_equal_to_zero!(self, max_acceleration_travel);
        setting_less_than_or_equal_to_zero!(self, max_acceleration_retracting);
        setting_less_than_or_equal_to_zero!(self, maximum_feedrate_x);
        setting_less_than_or_equal_to_zero!(self, maximum_feedrate_y);
        setting_less_than_or_equal_to_zero!(self, maximum_feedrate_z);
        setting_less_than_or_equal_to_zero!(self, maximum_feedrate_e);
        setting_less_than_zero!(self, number_of_perimeters);
        setting_less_than_zero!(self, infill_percentage);
        setting_less_than_zero!(self, top_layers);
        setting_less_than_zero!(self, bottom_layers);
        setting_less_than_zero!(self, retract_length);
        setting_less_than_zero!(self, retract_lift_z);
        setting_less_than_zero!(self, minimum_feedrate_travel);
        setting_less_than_zero!(self, minimum_feedrate_print);
        setting_less_than_zero!(self, minimum_retract_distance);

        if self.layer_height < self.nozzle_diameter * 0.2 {
            return SettingsValidationResult::Warning(SlicerWarnings::LayerSizeTooLow {
                layer_height: self.layer_height,
                nozzle_diameter: self.nozzle_diameter,
            });
        } else if self.layer_height > self.nozzle_diameter * 0.8 {
            return SettingsValidationResult::Warning(SlicerWarnings::LayerSizeTooHigh {
                layer_height: self.layer_height,
                nozzle_diameter: self.nozzle_diameter,
            });
        }

        let r = check_extrusions(&self.extrusion_width, self.nozzle_diameter);
        match r {
            SettingsValidationResult::NoIssue => {}
            _ => return r,
        }

        let r = check_accelerations(
            &self.acceleration,
            &self.speed,
            self.print_x.min(self.print_y),
        );
        match r {
            SettingsValidationResult::NoIssue => {}
            _ => return r,
        }

        if let Some(skirt) = self.skirt.as_ref() {
            if let Some(brim) = self.brim_width.as_ref() {
                if skirt.distance <= *brim {
                    return SettingsValidationResult::Warning(
                        SlicerWarnings::SkirtAndBrimOverlap {
                            skirt_distance: skirt.distance,
                            brim_width: *brim,
                        },
                    );
                }
            }
        }

        if self.filament.extruder_temp < 140.0 {
            return SettingsValidationResult::Warning(SlicerWarnings::NozzleTemperatureTooLow {
                temp: self.filament.extruder_temp,
            });
        } else if self.filament.extruder_temp > 260.0 {
            return SettingsValidationResult::Warning(SlicerWarnings::NozzleTemperatureTooHigh {
                temp: self.filament.extruder_temp,
            });
        }

        for (_, pls) in &self.layer_settings {
            option_setting_less_than_or_equal_to_zero!(pls, layer_height);
            option_setting_less_than_zero!(pls, infill_percentage);
            option_setting_less_than_zero!(pls, retraction_length);

            if let Some(layer_height) = pls.layer_height {
                if layer_height < self.nozzle_diameter * 0.2 {
                    return SettingsValidationResult::Warning(SlicerWarnings::LayerSizeTooLow {
                        layer_height: self.layer_height,
                        nozzle_diameter: self.nozzle_diameter,
                    });
                } else if layer_height > self.nozzle_diameter * 0.8 {
                    return SettingsValidationResult::Warning(SlicerWarnings::LayerSizeTooHigh {
                        layer_height: self.layer_height,
                        nozzle_diameter: self.nozzle_diameter,
                    });
                }
            }

            if let Some(extruder_temp) = pls.extruder_temp {
                if extruder_temp < 140.0 {
                    return SettingsValidationResult::Warning(
                        SlicerWarnings::NozzleTemperatureTooLow {
                            temp: self.filament.extruder_temp,
                        },
                    );
                } else if extruder_temp > 260.0 {
                    return SettingsValidationResult::Warning(
                        SlicerWarnings::NozzleTemperatureTooHigh {
                            temp: self.filament.extruder_temp,
                        },
                    );
                }
            }

            let r = if let Some(extrusion_width) = &pls.extrusion_width {
                check_extrusions(extrusion_width, self.nozzle_diameter)
            } else {
                SettingsValidationResult::NoIssue
            };

            match r {
                SettingsValidationResult::NoIssue => {}
                _ => return r,
            }

            let r = check_accelerations(
                pls.acceleration.as_ref().unwrap_or(&self.acceleration),
                pls.speed.as_ref().unwrap_or(&self.speed),
                self.print_x.min(self.print_y),
            );
            match r {
                SettingsValidationResult::NoIssue => {}
                _ => return r,
            }
        }

        SettingsValidationResult::NoIssue
    }
}

///Possible results of validation the settings
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum SettingsValidationResult {
    ///No Issue
    NoIssue,
    ///A warning
    Warning(SlicerWarnings),
    ///An error
    Error(SlicerErrors),
}

///Settings specific to a Layer
pub struct LayerSettings {
    ///The height of the layers
    pub layer_height: f64,

    ///Inset the layer by the provided amount, if None on inset will be performed
    pub layer_shrink_amount: Option<f64>,

    ///The speeds used for movement
    pub speed: MovementParameter,

    ///The acceleration for movement
    pub acceleration: MovementParameter,

    ///The extrusion width of the layers
    pub extrusion_width: MovementParameter,

    ///Solid Infill type
    pub solid_infill_type: SolidInfillTypes,

    ///Partial Infill type
    pub partial_infill_type: PartialInfillTypes,

    ///The percentage of infill to use for partial infill
    pub infill_percentage: f64,

    ///Overlap between infill and interior perimeters
    pub infill_perimeter_overlap_percentage: f64,

    ///Controls the order of perimeters
    pub inner_perimeters_first: bool,

    ///Temperature of the bed
    pub bed_temp: f64,

    ///Temperature of the extuder
    pub extruder_temp: f64,

    ///Retraction Wipe
    pub retraction_wipe: Option<RetractionWipeSettings>,

    ///Retraction Distance
    pub retraction_length: f64,
}

///A set of values for different movement types
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MovementParameter {
    ///Value for interior (perimeters that are inside the model
    pub interior_inner_perimeter: f64,

    /// Value for interior perimeters surface perimeter
    pub interior_surface_perimeter: f64,

    ///Value for exterior perimeters that are inside the model
    pub exterior_inner_perimeter: f64,

    ///Value for exterior surface perimeter
    pub exterior_surface_perimeter: f64,

    ///Value for solid top infill moves
    pub solid_top_infill: f64,

    ///Value for solid infill moves
    pub solid_infill: f64,

    ///Value for pertial infill moves
    pub infill: f64,

    ///Value for travel moves
    pub travel: f64,

    ///Value for bridging
    pub bridge: f64,

    ///Value for support structures
    pub support: f64,
}

impl MovementParameter {
    ///Returns the associated value to the move type provided
    pub fn get_value_for_movement_type(&self, move_type: &MoveType) -> f64 {
        match move_type {
            MoveType::TopSolidInfill => self.solid_top_infill,
            MoveType::SolidInfill => self.solid_infill,
            MoveType::Infill => self.infill,
            MoveType::ExteriorSurfacePerimeter => self.exterior_surface_perimeter,
            MoveType::InteriorSurfacePerimeter => self.interior_surface_perimeter,
            MoveType::ExteriorInnerPerimeter => self.exterior_inner_perimeter,
            MoveType::InteriorInnerPerimeter => self.interior_inner_perimeter,
            MoveType::Bridging => self.bridge,
            MoveType::Support => self.support,
            MoveType::Travel => self.travel,
        }
    }
}
///Settings for a filament
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct FilamentSettings {
    ///Diameter of this filament in mm
    pub diameter: f64,

    ///Density of this filament in grams per cm^3
    pub density: f64,

    ///Cost of this filament in $ per kg
    pub cost: f64,

    ///Extruder temp for this filament
    pub extruder_temp: f64,

    ///Bed temp for this filament
    pub bed_temp: f64,
}

///Settigns for the fans
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct FanSettings {
    ///The default fan speed
    pub fan_speed: f64,

    ///Disable the fan for layers below this value
    pub disable_fan_for_layers: usize,

    ///Threshold to start slowing down based on layer print time in seconds
    pub slow_down_threshold: f64,

    ///Minimum speed to slow down to
    pub min_print_speed: f64,
}

impl Default for FilamentSettings {
    fn default() -> Self {
        FilamentSettings {
            diameter: 1.75,
            density: 1.24,
            cost: 24.99,
            extruder_temp: 210.0,
            bed_temp: 60.0,
        }
    }
}

impl Default for FanSettings {
    fn default() -> Self {
        FanSettings {
            fan_speed: 100.0,
            disable_fan_for_layers: 1,
            slow_down_threshold: 15.0,
            min_print_speed: 15.0,
        }
    }
}

///Support settings
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SupportSettings {
    ///Angle to start production supports in degrees
    pub max_overhang_angle: f64,

    ///Spacing between the ribs of support
    pub support_spacing: f64,
}

///The Settings for Skirt generation
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SkirtSettings {
    ///the number of layer to generate the skirt
    pub layers: usize,

    ///Distance from the models to place the skirt
    pub distance: f64,
}

///The Settings for Skirt generation
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RetractionWipeSettings {
    ///The speed the retract wipe move
    pub speed: f64,

    ///The acceleration the retract wipe move
    pub acceleration: f64,

    ///Wipe Distance in mm
    pub distance: f64,
}

///A partial complete settings file
#[derive(Serialize, Deserialize, Debug)]
pub struct PartialSettings {
    ///The height of the layers
    pub layer_height: Option<f64>,

    ///The extrusion width of the layers
    pub extrusion_width: Option<MovementParameter>,

    ///Inset the layer by the provided amount, if None on inset will be performed
    pub layer_shrink_amount: Option<f64>,
    ///The filament Settings
    pub filament: Option<FilamentSettings>,
    ///The fan settings
    pub fan: Option<FanSettings>,
    ///The skirt settings, if None no skirt will be generated
    pub skirt: Option<SkirtSettings>,
    ///The support settings, if None no support will be generated
    pub support: Option<SupportSettings>,
    ///Diameter of the nozzle in mm
    pub nozzle_diameter: Option<f64>,

    ///length to retract in mm
    pub retract_length: Option<f64>,

    ///Retraction Wipe
    pub retraction_wipe: Option<RetractionWipeSettings>,

    ///Distance to lift the z axis during a retract
    pub retract_lift_z: Option<f64>,

    ///The velocity of retracts
    pub retract_speed: Option<f64>,

    ///The speeds used for movement
    pub speed: Option<MovementParameter>,

    ///The acceleration for movement
    pub acceleration: Option<MovementParameter>,

    ///The percentage of infill to use for partial infill
    pub infill_percentage: Option<f64>,

    ///Controls the order of perimeters
    pub inner_perimeters_first: Option<bool>,

    ///Number of perimeters to use if possible
    pub number_of_perimeters: Option<usize>,

    ///Number of solid top layers before infill
    pub top_layers: Option<usize>,

    ///Number of solid bottom layers before infill
    pub bottom_layers: Option<usize>,

    ///Size of the printer in x dimension in mm
    pub print_x: Option<f64>,

    ///Size of the printer in y dimension in mm
    pub print_y: Option<f64>,

    ///Size of the printer in z dimension in mm
    pub print_z: Option<f64>,

    ///Width of the brim, if None no brim will be generated
    pub brim_width: Option<f64>,

    ///The minimum travel distance required to perform a retraction
    pub minimum_retract_distance: Option<f64>,

    ///Overlap between infill and interior perimeters
    pub infill_perimeter_overlap_percentage: Option<f64>,

    ///Solid Infill type
    pub solid_infill_type: Option<SolidInfillTypes>,

    ///Partial Infill type
    pub partial_infill_type: Option<PartialInfillTypes>,

    ///The instructions to prepend to the exported instructions
    pub starting_instructions: Option<String>,

    ///The instructions to append to the end of the exported instructions
    pub ending_instructions: Option<String>,

    /// The instructions to append before layer changes
    pub before_layer_change_instructions: Option<String>,

    /// The instructions to append after layer changes
    pub after_layer_change_instructions: Option<String>,

    /// The instructions to append between object changes
    pub object_change_instructions: Option<String>,

    ///Other files to load
    pub other_files: Option<Vec<String>>,

    ///Maximum Acceleration in x dimension
    pub max_acceleration_x: Option<f64>,
    ///Maximum Acceleration in y dimension
    pub max_acceleration_y: Option<f64>,
    ///Maximum Acceleration in z dimension
    pub max_acceleration_z: Option<f64>,
    ///Maximum Acceleration in e dimension
    pub max_acceleration_e: Option<f64>,

    ///Maximum Acceleration while extruding
    pub max_acceleration_extruding: Option<f64>,
    ///Maximum Acceleration while traveling
    pub max_acceleration_travel: Option<f64>,
    ///Maximum Acceleration while retracting
    pub max_acceleration_retracting: Option<f64>,

    ///Maximum Jerk in x dimension
    pub max_jerk_x: Option<f64>,
    ///Maximum Jerk in y dimension
    pub max_jerk_y: Option<f64>,
    ///Maximum Jerk in z dimension
    pub max_jerk_z: Option<f64>,
    ///Maximum Jerk in e dimension
    pub max_jerk_e: Option<f64>,

    ///Minimum feedrate for extrusion moves
    pub minimum_feedrate_print: Option<f64>,
    ///Minimum feedrate for travel moves
    pub minimum_feedrate_travel: Option<f64>,
    ///Maximum feedrate for x dimension
    pub maximum_feedrate_x: Option<f64>,
    ///Maximum feedrate for y dimension
    pub maximum_feedrate_y: Option<f64>,
    ///Maximum feedrate for z dimension
    pub maximum_feedrate_z: Option<f64>,
    ///Maximum feedrate for e dimension
    pub maximum_feedrate_e: Option<f64>,

    ///Settings for specific layers
    pub layer_settings: Option<Vec<(LayerRange, PartialLayerSettings)>>,
}

impl PartialSettings {
    ///Convert a partial settings file into a complete settings file
    /// returns an error if a settings is not present in this or any sub file
    pub fn get_settings(mut self) -> Result<Settings, SlicerErrors> {
        self.combine_with_other_files()?;

        try_convert_partial_to_settings(self).map_err(|err| {
            SlicerErrors::SettingsFileMissingSettings {
                missing_setting: err,
            }
        })
    }

    fn combine_with_other_files(&mut self) -> Result<(), SlicerErrors> {
        let files: Vec<String> = self
            .other_files
            .as_mut()
            .map(|of| of.drain(..).collect())
            .unwrap_or_default();

        for file in &files {
            let mut ps: PartialSettings =
                deser_hjson::from_str(&std::fs::read_to_string(file).map_err(|_| {
                    SlicerErrors::SettingsRecursiveLoadError {
                        filepath: file.to_string(),
                    }
                })?)
                .map_err(|_| SlicerErrors::SettingsFileMisformat {
                    filepath: file.to_string(),
                })?;

            ps.combine_with_other_files()?;

            *self = self.combine(ps);
        }

        Ok(())
    }

    fn combine(&self, other: PartialSettings) -> PartialSettings {
        PartialSettings {
            layer_height: self.layer_height.or(other.layer_height),
            extrusion_width: self
                .extrusion_width
                .clone()
                .or_else(|| other.extrusion_width.clone()),
            layer_shrink_amount: self.layer_shrink_amount.or(other.layer_shrink_amount),
            filament: self.filament.clone().or_else(|| other.filament.clone()),
            fan: self.fan.clone().or_else(|| other.fan.clone()),
            skirt: self.skirt.clone().or_else(|| other.skirt.clone()),
            support: self.support.clone().or_else(|| other.support.clone()),
            nozzle_diameter: self.nozzle_diameter.or(other.nozzle_diameter),
            retract_length: self.retract_length.or(other.retract_length),
            retraction_wipe: self.retraction_wipe.clone().or(other.retraction_wipe),
            retract_lift_z: self.retract_lift_z.or(other.retract_lift_z),
            retract_speed: self.retract_speed.or(other.retract_speed),
            speed: self.speed.clone().or_else(|| other.speed.clone()),
            acceleration: self
                .acceleration
                .clone()
                .or_else(|| other.acceleration.clone()),
            infill_percentage: self.infill_percentage.or(other.infill_percentage),
            inner_perimeters_first: self.inner_perimeters_first.or(other.inner_perimeters_first),
            number_of_perimeters: self.number_of_perimeters.or(other.number_of_perimeters),
            top_layers: self.top_layers.or(other.top_layers),
            bottom_layers: self.bottom_layers.or(other.bottom_layers),
            print_x: self.print_x.or(other.print_x),
            print_y: self.print_y.or(other.print_y),
            print_z: self.print_z.or(other.print_z),
            brim_width: self.brim_width.or(other.brim_width),
            minimum_retract_distance: self
                .minimum_retract_distance
                .or(other.minimum_retract_distance),
            infill_perimeter_overlap_percentage: self
                .infill_perimeter_overlap_percentage
                .or(other.infill_perimeter_overlap_percentage),
            solid_infill_type: self.solid_infill_type.or(other.solid_infill_type),
            partial_infill_type: self.partial_infill_type.or(other.partial_infill_type),
            starting_instructions: self
                .starting_instructions
                .clone()
                .or_else(|| other.starting_instructions.clone()),
            ending_instructions: self
                .ending_instructions
                .clone()
                .or(other.ending_instructions),
            before_layer_change_instructions: self
                .before_layer_change_instructions
                .clone()
                .or(other.before_layer_change_instructions),
            after_layer_change_instructions: self
                .after_layer_change_instructions
                .clone()
                .or(other.after_layer_change_instructions),
            object_change_instructions: self
                .object_change_instructions
                .clone()
                .or(other.object_change_instructions),
            other_files: None,
            max_acceleration_x: self.max_acceleration_x.or(other.max_acceleration_x),
            max_acceleration_y: self.max_acceleration_y.or(other.max_acceleration_y),
            max_acceleration_z: self.max_acceleration_z.or(other.max_acceleration_z),
            max_acceleration_e: self.max_acceleration_e.or(other.max_acceleration_e),
            max_acceleration_extruding: self
                .max_acceleration_extruding
                .or(other.max_acceleration_extruding),
            max_acceleration_travel: self
                .max_acceleration_travel
                .or(other.max_acceleration_travel),
            max_acceleration_retracting: self
                .max_acceleration_retracting
                .or(other.max_acceleration_retracting),
            max_jerk_x: self.max_jerk_x.or(other.max_jerk_x),
            max_jerk_y: self.max_jerk_y.or(other.max_jerk_y),
            max_jerk_z: self.max_jerk_z.or(other.max_jerk_z),
            max_jerk_e: self.max_jerk_e.or(other.max_jerk_e),
            minimum_feedrate_print: self.minimum_feedrate_print.or(other.minimum_feedrate_print),
            minimum_feedrate_travel: self
                .minimum_feedrate_travel
                .or(other.minimum_feedrate_travel),
            maximum_feedrate_x: self.maximum_feedrate_x.or(other.maximum_feedrate_x),
            maximum_feedrate_y: self.maximum_feedrate_y.or(other.maximum_feedrate_y),
            maximum_feedrate_z: self.maximum_feedrate_z.or(other.maximum_feedrate_z),
            maximum_feedrate_e: self.maximum_feedrate_e.or(other.maximum_feedrate_e),
            layer_settings: {
                match (self.layer_settings.as_ref(), other.layer_settings.as_ref()) {
                    (None, None) => None,
                    (None, Some(v)) | (Some(v), None) => Some(v.clone()),
                    (Some(a), Some(b)) => {
                        let mut v = vec![];
                        v.append(&mut a.clone());
                        v.append(&mut b.clone());
                        Some(v)
                    }
                }
            },
        }
    }
}

/// The different types of layer ranges supported
#[derive(Deserialize, Serialize, Debug, Clone)]
pub enum LayerRange {
    ///A single single based on the index
    SingleLayer(usize),

    ///A range of layers based on index inclusive
    LayerCountRange {
        ///The start index
        start: usize,

        ///The end index
        end: usize,
    },

    ///A Range of layers based on the height of the bottom on the slice
    HeightRange {
        ///The start height
        start: f64,

        ///The end height
        end: f64,
    },
}

///A Partial List of all slicer settings
#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct PartialLayerSettings {
    ///The height of the layers
    pub layer_height: Option<f64>,

    ///Inset the layer by the provided amount, if None on inset will be performed
    pub layer_shrink_amount: Option<f64>,

    ///The speeds used for movement
    pub speed: Option<MovementParameter>,

    ///The acceleration for movement
    pub acceleration: Option<MovementParameter>,

    ///The extrusion widths of the layers
    pub extrusion_width: Option<MovementParameter>,

    ///Solid Infill type
    pub solid_infill_type: Option<SolidInfillTypes>,

    ///Partial Infill type
    pub partial_infill_type: Option<PartialInfillTypes>,

    ///The percentage of infill to use for partial infill
    pub infill_percentage: Option<f64>,

    ///Overlap between infill and interior perimeters
    pub infill_perimeter_overlap_percentage: Option<f64>,

    ///Controls the order of perimeters
    pub inner_perimeters_first: Option<bool>,

    ///The Bed Temperature
    pub bed_temp: Option<f64>,

    ///The Extruder Temperature
    pub extruder_temp: Option<f64>,

    ///Retraction Wipe
    pub retraction_wipe: Option<RetractionWipeSettings>,

    ///Retraction Distance
    pub retraction_length: Option<f64>,
}

impl PartialLayerSettings {
    fn combine(&self, other: &PartialLayerSettings) -> PartialLayerSettings {
        PartialLayerSettings {
            layer_height: self.layer_height.or(other.layer_height),
            extrusion_width: self
                .extrusion_width
                .clone()
                .or_else(|| other.extrusion_width.clone()),
            speed: self.speed.clone().or_else(|| other.speed.clone()),
            acceleration: self
                .acceleration
                .clone()
                .or_else(|| other.acceleration.clone()),
            infill_percentage: self.infill_percentage.or(other.infill_percentage),

            inner_perimeters_first: self.inner_perimeters_first.or(other.inner_perimeters_first),

            bed_temp: self.bed_temp.or(other.bed_temp),
            extruder_temp: self.extruder_temp.or(other.extruder_temp),
            retraction_wipe: self
                .retraction_wipe
                .clone()
                .or_else(|| other.retraction_wipe.clone()),
            infill_perimeter_overlap_percentage: self
                .infill_perimeter_overlap_percentage
                .or(other.infill_perimeter_overlap_percentage),
            solid_infill_type: self.solid_infill_type.or(other.solid_infill_type),
            partial_infill_type: self.partial_infill_type.or(other.partial_infill_type),
            layer_shrink_amount: self.layer_shrink_amount.or(other.layer_shrink_amount),
            retraction_length: self.retraction_length.or(other.retraction_length),
        }
    }
}

fn try_convert_partial_to_settings(part: PartialSettings) -> Result<Settings, String> {
    Ok(Settings {
        layer_height: part.layer_height.ok_or("layer_height")?,
        extrusion_width: part.extrusion_width.ok_or("extrusion_width")?,
        filament: part.filament.ok_or("filament")?,
        fan: part.fan.ok_or("fan")?,
        skirt: part.skirt,
        support: part.support,
        nozzle_diameter: part.nozzle_diameter.ok_or("nozzle_diameter")?,
        retract_length: part.retract_length.ok_or("retract_length")?,
        retract_lift_z: part.retract_lift_z.ok_or("retract_lift_z")?,
        retract_speed: part.retract_speed.ok_or("retract_speed")?,
        retraction_wipe: part.retraction_wipe,
        speed: part.speed.ok_or("speed")?,
        acceleration: part.acceleration.ok_or("acceleration")?,
        infill_percentage: part.infill_percentage.ok_or("infill_percentage")?,
        inner_perimeters_first: part
            .inner_perimeters_first
            .ok_or("inner_perimeters_first")?,
        number_of_perimeters: part.number_of_perimeters.ok_or("number_of_perimeters")?,
        top_layers: part.top_layers.ok_or("top_layers")?,
        bottom_layers: part.bottom_layers.ok_or("bottom_layers")?,
        print_x: part.print_x.ok_or("print_x")?,
        print_y: part.print_y.ok_or("print_y")?,
        print_z: part.print_z.ok_or("print_z")?,
        brim_width: part.brim_width,
        layer_shrink_amount: part.layer_shrink_amount,
        minimum_retract_distance: part
            .minimum_retract_distance
            .ok_or("minimum_retract_distance")?,
        infill_perimeter_overlap_percentage: part
            .infill_perimeter_overlap_percentage
            .ok_or("infill_perimeter_overlap_percentage")?,
        solid_infill_type: part.solid_infill_type.ok_or("solid_infill_type")?,
        partial_infill_type: part.partial_infill_type.ok_or("partial_infill_type")?,
        starting_instructions: part.starting_instructions.ok_or("starting_instructions")?,
        ending_instructions: part.ending_instructions.ok_or("ending_instructions")?,
        before_layer_change_instructions: part
            .before_layer_change_instructions
            .ok_or("before_layer_change_instructions")?,
        after_layer_change_instructions: part
            .after_layer_change_instructions
            .ok_or("after_layer_change_instructions")?,
        object_change_instructions: part
            .object_change_instructions
            .ok_or("object_change_instructions")?,

        max_acceleration_x: part.max_acceleration_x.ok_or("max_acceleration_x")?,
        max_acceleration_y: part.max_acceleration_y.ok_or("max_acceleration_y")?,
        max_acceleration_z: part.max_acceleration_z.ok_or("max_acceleration_z")?,
        max_acceleration_e: part.max_acceleration_e.ok_or("max_acceleration_e")?,
        max_acceleration_extruding: part
            .max_acceleration_extruding
            .ok_or("max_acceleration_extruding")?,
        max_acceleration_travel: part
            .max_acceleration_travel
            .ok_or("max_acceleration_travel")?,
        max_acceleration_retracting: part
            .max_acceleration_retracting
            .ok_or("max_acceleration_retracting")?,
        max_jerk_x: part.max_jerk_x.ok_or("max_jerk_x")?,
        max_jerk_y: part.max_jerk_y.ok_or("max_jerk_y")?,
        max_jerk_z: part.max_jerk_z.ok_or("max_jerk_z")?,
        max_jerk_e: part.max_jerk_e.ok_or("max_jerk_e")?,
        minimum_feedrate_print: part
            .minimum_feedrate_print
            .ok_or("minimum_feedrate_print")?,
        minimum_feedrate_travel: part
            .minimum_feedrate_travel
            .ok_or("minimum_feedrate_travel")?,
        maximum_feedrate_x: part.maximum_feedrate_x.ok_or("maximum_feedrate_x")?,
        maximum_feedrate_y: part.maximum_feedrate_y.ok_or("maximum_feedrate_y")?,
        maximum_feedrate_z: part.maximum_feedrate_z.ok_or("maximum_feedrate_z")?,
        maximum_feedrate_e: part.maximum_feedrate_e.ok_or("maximum_feedrate_e")?,
        layer_settings: part.layer_settings.unwrap_or_default(),
    })
}

fn check_extrusions(
    extrusion_width: &MovementParameter,
    nozzle_diameter: f64,
) -> SettingsValidationResult {
    //infill
    if extrusion_width.infill < nozzle_diameter * 0.6 {
        return SettingsValidationResult::Warning(SlicerWarnings::ExtrusionWidthTooLow {
            extrusion_width: extrusion_width.infill,
            nozzle_diameter,
        });
    } else if extrusion_width.infill > nozzle_diameter * 2.0 {
        return SettingsValidationResult::Warning(SlicerWarnings::ExtrusionWidthTooHigh {
            extrusion_width: extrusion_width.infill,
            nozzle_diameter,
        });
    }

    //top infill
    if extrusion_width.solid_top_infill < nozzle_diameter * 0.6 {
        return SettingsValidationResult::Warning(SlicerWarnings::ExtrusionWidthTooLow {
            extrusion_width: extrusion_width.solid_top_infill,
            nozzle_diameter,
        });
    } else if extrusion_width.solid_top_infill > nozzle_diameter * 2.0 {
        return SettingsValidationResult::Warning(SlicerWarnings::ExtrusionWidthTooHigh {
            extrusion_width: extrusion_width.solid_top_infill,
            nozzle_diameter,
        });
    }

    //solid infill
    if extrusion_width.solid_infill < nozzle_diameter * 0.6 {
        return SettingsValidationResult::Warning(SlicerWarnings::ExtrusionWidthTooLow {
            extrusion_width: extrusion_width.solid_infill,
            nozzle_diameter,
        });
    } else if extrusion_width.solid_infill > nozzle_diameter * 2.0 {
        return SettingsValidationResult::Warning(SlicerWarnings::ExtrusionWidthTooHigh {
            extrusion_width: extrusion_width.solid_infill,
            nozzle_diameter,
        });
    }

    //bridge
    if extrusion_width.bridge < nozzle_diameter * 0.6 {
        return SettingsValidationResult::Warning(SlicerWarnings::ExtrusionWidthTooLow {
            extrusion_width: extrusion_width.bridge,
            nozzle_diameter,
        });
    } else if extrusion_width.bridge > nozzle_diameter * 2.0 {
        return SettingsValidationResult::Warning(SlicerWarnings::ExtrusionWidthTooHigh {
            extrusion_width: extrusion_width.bridge,
            nozzle_diameter,
        });
    }

    //support
    if extrusion_width.support < nozzle_diameter * 0.6 {
        return SettingsValidationResult::Warning(SlicerWarnings::ExtrusionWidthTooLow {
            extrusion_width: extrusion_width.support,
            nozzle_diameter,
        });
    } else if extrusion_width.support > nozzle_diameter * 2.0 {
        return SettingsValidationResult::Warning(SlicerWarnings::ExtrusionWidthTooHigh {
            extrusion_width: extrusion_width.support,
            nozzle_diameter,
        });
    }

    //interior_surface_perimeter
    if extrusion_width.interior_surface_perimeter < nozzle_diameter * 0.6 {
        return SettingsValidationResult::Warning(SlicerWarnings::ExtrusionWidthTooLow {
            extrusion_width: extrusion_width.interior_surface_perimeter,
            nozzle_diameter,
        });
    } else if extrusion_width.interior_surface_perimeter > nozzle_diameter * 2.0 {
        return SettingsValidationResult::Warning(SlicerWarnings::ExtrusionWidthTooHigh {
            extrusion_width: extrusion_width.interior_surface_perimeter,
            nozzle_diameter,
        });
    }

    //interior_inner_perimeter
    if extrusion_width.interior_inner_perimeter < nozzle_diameter * 0.6 {
        return SettingsValidationResult::Warning(SlicerWarnings::ExtrusionWidthTooLow {
            extrusion_width: extrusion_width.interior_inner_perimeter,
            nozzle_diameter,
        });
    } else if extrusion_width.interior_inner_perimeter > nozzle_diameter * 2.0 {
        return SettingsValidationResult::Warning(SlicerWarnings::ExtrusionWidthTooHigh {
            extrusion_width: extrusion_width.interior_inner_perimeter,
            nozzle_diameter,
        });
    }

    //exterior_inner_perimeter
    if extrusion_width.exterior_inner_perimeter < nozzle_diameter * 0.6 {
        return SettingsValidationResult::Warning(SlicerWarnings::ExtrusionWidthTooLow {
            extrusion_width: extrusion_width.exterior_inner_perimeter,
            nozzle_diameter,
        });
    } else if extrusion_width.exterior_inner_perimeter > nozzle_diameter * 2.0 {
        return SettingsValidationResult::Warning(SlicerWarnings::ExtrusionWidthTooHigh {
            extrusion_width: extrusion_width.exterior_inner_perimeter,
            nozzle_diameter,
        });
    }

    //exterior_surface_perimeter
    if extrusion_width.exterior_surface_perimeter < nozzle_diameter * 0.6 {
        return SettingsValidationResult::Warning(SlicerWarnings::ExtrusionWidthTooLow {
            extrusion_width: extrusion_width.exterior_surface_perimeter,
            nozzle_diameter,
        });
    } else if extrusion_width.exterior_surface_perimeter > nozzle_diameter * 2.0 {
        return SettingsValidationResult::Warning(SlicerWarnings::ExtrusionWidthTooHigh {
            extrusion_width: extrusion_width.exterior_surface_perimeter,
            nozzle_diameter,
        });
    }

    SettingsValidationResult::NoIssue
}

fn check_accelerations(
    acceleration: &MovementParameter,
    speed: &MovementParameter,
    min_bed_dimension: f64,
) -> SettingsValidationResult {
    //infill
    if (speed.infill * speed.infill) / (2.0 * acceleration.infill) > min_bed_dimension {
        return SettingsValidationResult::Warning(SlicerWarnings::AccelerationTooLow {
            acceleration: acceleration.infill,
            speed: speed.infill,
            bed_size: min_bed_dimension,
        });
    }

    //top infill
    if (speed.solid_top_infill * speed.solid_top_infill) / (2.0 * acceleration.solid_top_infill)
        > min_bed_dimension
    {
        return SettingsValidationResult::Warning(SlicerWarnings::AccelerationTooLow {
            acceleration: acceleration.solid_top_infill,
            speed: speed.solid_top_infill,
            bed_size: min_bed_dimension,
        });
    }

    //solid infill
    if (speed.solid_infill * speed.solid_infill) / (2.0 * acceleration.solid_infill)
        > min_bed_dimension
    {
        return SettingsValidationResult::Warning(SlicerWarnings::AccelerationTooLow {
            acceleration: acceleration.solid_infill,
            speed: speed.solid_infill,
            bed_size: min_bed_dimension,
        });
    }

    //bridge
    if (speed.bridge * speed.bridge) / (2.0 * acceleration.bridge) > min_bed_dimension {
        return SettingsValidationResult::Warning(SlicerWarnings::AccelerationTooLow {
            acceleration: acceleration.bridge,
            speed: speed.bridge,
            bed_size: min_bed_dimension,
        });
    }

    //support
    if (speed.support * speed.support) / (2.0 * acceleration.support) > min_bed_dimension {
        return SettingsValidationResult::Warning(SlicerWarnings::AccelerationTooLow {
            acceleration: acceleration.support,
            speed: speed.support,
            bed_size: min_bed_dimension,
        });
    }

    //interior_surface_perimeter
    if (speed.interior_surface_perimeter * speed.interior_surface_perimeter)
        / (2.0 * acceleration.interior_surface_perimeter)
        > min_bed_dimension
    {
        return SettingsValidationResult::Warning(SlicerWarnings::AccelerationTooLow {
            acceleration: acceleration.interior_surface_perimeter,
            speed: speed.interior_surface_perimeter,
            bed_size: min_bed_dimension,
        });
    }

    //interior_inner_perimeter
    if (speed.interior_inner_perimeter * speed.interior_inner_perimeter)
        / (2.0 * acceleration.interior_inner_perimeter)
        > min_bed_dimension
    {
        return SettingsValidationResult::Warning(SlicerWarnings::AccelerationTooLow {
            acceleration: acceleration.interior_inner_perimeter,
            speed: speed.interior_inner_perimeter,
            bed_size: min_bed_dimension,
        });
    }

    //exterior_inner_perimeter
    if (speed.exterior_inner_perimeter * speed.exterior_inner_perimeter)
        / (2.0 * acceleration.exterior_inner_perimeter)
        > min_bed_dimension
    {
        return SettingsValidationResult::Warning(SlicerWarnings::AccelerationTooLow {
            acceleration: acceleration.exterior_inner_perimeter,
            speed: speed.exterior_inner_perimeter,
            bed_size: min_bed_dimension,
        });
    }

    //exterior_surface_perimeter
    if (speed.exterior_surface_perimeter * speed.exterior_surface_perimeter)
        / (2.0 * acceleration.exterior_surface_perimeter)
        > min_bed_dimension
    {
        return SettingsValidationResult::Warning(SlicerWarnings::AccelerationTooLow {
            acceleration: acceleration.exterior_surface_perimeter,
            speed: speed.exterior_surface_perimeter,
            bed_size: min_bed_dimension,
        });
    }

    SettingsValidationResult::NoIssue
}
