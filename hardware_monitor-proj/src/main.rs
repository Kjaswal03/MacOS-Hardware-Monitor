use core::f64;
use std::time::{Duration, Instant};
use std::collections::{VecDeque, HashMap};
use egui::RichText; 
use sysinfo::{System, SystemExt, CpuExt,ComponentExt, DiskExt};
use egui::{emath::Numeric, Color32};
use eframe::{self, egui};
use egui_plot::{Plot, Line};


struct MyApp {
    sys_ovr_info: String,
    cpu_usage_info: String,
    ram_usage_info: String,
    sensor_temp_info: String,
    disk_data_info: String,
    last_update_time: Instant,
    ram_history: VecDeque<f64>,
    cpu_history: VecDeque<f64>,
    sensor_history: HashMap<String, VecDeque<f64>>,
    last_update: Instant,
}

impl Default for MyApp {
    fn default() -> Self {
        Self {
            sys_ovr_info: String::from("System info will be displayed here."),
            cpu_usage_info: String::from("CPU usage information will be displayed here."),
            ram_usage_info: String::from("RAM usage information will be displayed here."),
            sensor_temp_info: String::from("CPU die Temp information will be displayed here."),
            disk_data_info: String::from("Disk data info will be displayed here."),
            last_update_time: Instant::now(),
            ram_history: VecDeque::with_capacity(100),
            cpu_history: VecDeque::with_capacity(100),
            sensor_history: HashMap::new(),
            last_update: Instant::now(), 

        }
    }
}



impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {

        let current_time = Instant::now();

        // Optimization for updating data (expensive call if called in every data function)
        let mut sys = System::new_all();
        sys.refresh_all();

        self.sys_ovr_info = sys_info(&sys);
        
        // Check if one second has passed since the last update
        if current_time.duration_since(self.last_update) >= Duration::from_secs(1) {
            // Refresh hardware data 
            self.cpu_usage_info = cpu_usage(self, &sys);
            self.ram_usage_info = ram_info(self, &sys);
            self.sensor_temp_info = sensor_info(self,&sys);
            self.disk_data_info = disk_info(&sys);
            self.last_update_time = current_time;

            // Reset the last update time
            self.last_update = current_time;
        }
        
        
        egui::CentralPanel::default().show(ctx, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui|{

                // Setting Dimensions for the Graph
                let screen_size = ctx.available_rect().size();
                let plot_width = screen_size.x * 0.9;
                let plot_height = screen_size.y * 0.2;

                ui.label(RichText::new("macOS Hardware Monitor").strong().color(Color32::WHITE).size(50.0));

                // Button to allow users to manually update the data
                if ui.button("Refresh Data").clicked() {
                    self.cpu_usage_info = cpu_usage(self, &sys);
                    self.ram_usage_info = ram_info(self, &sys);
                    self.sensor_temp_info = sensor_info(self,&sys);
                    self.disk_data_info = disk_info(&sys);
                    self.last_update = Instant::now();
                }

                ui.label(RichText::new(&self.sys_ovr_info).strong().color(Color32::GOLD));

                // CPU Usage Utilzation + CPU Usage Utilization Graph
                ui.label(RichText::new(&self.cpu_usage_info).strong().color(Color32::GOLD));
                ui.spacing();
                ui.spacing();
                ui.label(RichText::new("Average CPU Utilization Usage Graph: (Time,Percentage)").strong().underline());
                ui.spacing();

                // Collecting data points for plotting
                let cpu_points: Vec<[f64; 2]> = self.cpu_history.iter().enumerate().map(|(i, &value)| {
                    [i as f64, value]
                }).collect();


                // Creating the Graph
                let cpu_usage_line = Line::new(cpu_points);
                let cpu_usage_plot = Plot::new("cpu_usage_plot")
                    .width(plot_width)
                    .height(plot_height)
                    .view_aspect(2.0);
                cpu_usage_plot.show(ui, |plot_ui| plot_ui.line(cpu_usage_line));
                

                // Ram Usage + Ram Usage Graph
                ui.label(RichText::new(&self.ram_usage_info).strong().color(Color32::GOLD));
                ui.spacing();
                ui.spacing();
                ui.label(RichText::new("Ram Usage Graph: (Time,Percentage)").strong().underline());
                ui.spacing();

                // Collecting data points for plotting
                let r_data_points: Vec<[f64; 2]> = self.ram_history.iter().enumerate().map(|(i, &value)| {
                    [i as f64, value]
                }).collect();

                // Creating the Graph
                let ram_usage_line = Line::new(r_data_points);
                let ram_usage_plot = Plot::new("ram_usage_plot").view_aspect(2.0)
                    .width(plot_width)
                    .height(plot_height)
                    .view_aspect(2.0);
                ram_usage_plot.show(ui, |plot_ui| plot_ui.line(ram_usage_line));  

                // Sensor Data + Sensor Temp Graph
                ui.label(RichText::new(&self.sensor_temp_info).strong().color(Color32::GOLD));
                ui.spacing();
                ui.spacing();
                ui.label(RichText::new("Sensors Temps Graph: (Time,Temp)").strong().underline());
                ui.spacing();

                let mut sensor_data: Vec<(&String, &VecDeque<f64>)> = self.sensor_history.iter().collect();
            
                // Sort data shown (graphs) by 'tdie' number
                sensor_data.sort_by(|(a_label, _), (b_label, _)| {
                    let a_num = a_label.split("tdie").nth(1).unwrap_or("0").trim().parse::<i32>().unwrap_or(0);
                    let b_num = b_label.split("tdie").nth(1).unwrap_or("0").trim().parse::<i32>().unwrap_or(0);
                    a_num.cmp(&b_num)
                });

                for (sensor, sensor_history) in sensor_data {
                    // Unique Plot ID
                    let plot_id = format!("{}", sensor);

                    // Collecting data points for plotting
                    let sensor_data_points: Vec<[f64; 2]> = sensor_history.iter().enumerate().map(|(i, &value)| {
                        [i as f64, value]
                    }).collect();

                    // Creating the Graph
                    let sensor_temp_line = Line::new(sensor_data_points).name(sensor);
                    let sensor_usage_plot = Plot::new(plot_id).view_aspect(2.0)
                        .width(plot_width)
                        .height(plot_height)
                        .view_aspect(2.0);
                    sensor_usage_plot.show(ui, |plot_ui| plot_ui.line(sensor_temp_line));
                    ui.label(RichText::new(sensor).strong().color(Color32::RED).underline());
                    // Adding Spacing between each graph per sensor
                    ui.add_space(20.0);
                }

                // Disk Usage Data
                ui.label(RichText::new(&self.disk_data_info).strong().color(Color32::GOLD));
            
            });
        });
    }
}

fn main() -> Result<(), eframe::Error> {  
    let options = eframe::NativeOptions {
        initial_window_size: Some(egui::vec2(1500.0, 1000.0)),
        ..Default::default()
    };

    // Define a function that matches the expected signature
    fn create_app(_ctx: &eframe::CreationContext) -> Box<dyn eframe::App> {
        Box::new(MyApp::default())
    }

    eframe::run_native("Hardware Monitor", options, Box::new(create_app))
}


// Useful Hardware Data Functions
fn sys_info(sys: &System) -> String {

    // Data about System Information

    let mut sys_ovr_string = String::from("\n=> System Information:\n");

    sys_ovr_string.push_str(&format!("System name:                   {:?}\n",sys.name()));
    sys_ovr_string.push_str(&format!("System kernel version:    {:?}\n",sys.kernel_version()));
    sys_ovr_string.push_str(&format!("System OS version:          {:?}\n",sys.os_version()));
    sys_ovr_string.push_str(&format!("System host name:          {:?}",sys.host_name()));

    sys_ovr_string

}


fn cpu_usage(app: &mut MyApp, sys: &System) -> String {
 
    // Data about CPU usage
    let mut total_usage: f64 = 0.0;
    let avg : f64;

    let mut cpu_usage_string = String::from("\n=> CPU Usage:\n");

    for core in sys.cpus() {
        total_usage = total_usage + core.cpu_usage().to_f64();
        cpu_usage_string.push_str(&format!("Core {}: {:.2}% Frequency: {}Ghz\n", core.name(), core.cpu_usage(), core.frequency()));
    }

    avg = total_usage / sys.cpus().len().to_f64();

    // Update CPU usage history
    if app.cpu_history.len() >= 100 {
        app.cpu_history.pop_front();
    }
    app.cpu_history.push_back(avg);

    cpu_usage_string.push_str(&format!("Avg usage: {:.2}%", &avg.to_string()));
    cpu_usage_string

}

fn ram_info(app: &mut MyApp, sys: &System) -> String{

    // Data about RAM
    // Conversion to Gibibytes
    let total_mem = sys.total_memory().to_f64() * 9.31323E-10;
    let used_mem = sys.used_memory().to_f64() * 9.31323E-10;   
    let mem_usage: f64 = (used_mem / total_mem) * 100.0;

        // Update RAM usage history
        if app.ram_history.len() >= 100 {
            app.ram_history.pop_front();
        }
        app.ram_history.push_back(mem_usage);
        

    let mut ram_usage_string = String::from("=> RAM Usage:\n");

    ram_usage_string.push_str(&format!("In use: {:.3} GB\nTotal: {:.2} GB\nMemory: {:.2}%", 
    &used_mem.to_string(), 
    &total_mem.to_string(), 
    &mem_usage.to_string()
    ));
    ram_usage_string
}


fn sensor_info(app: &mut MyApp, sys: &System) -> String {
    let mut sensor_temp_string = String::from("\n=> CPU die Temperatures:\n");
    let mut sensors: Vec<(&str, f64)> = Vec::new();

    // Data about Temps of CPU die
    for sensor in sys.components() {
        if sensor.label().starts_with("PMU tdie")  {
            let tdie_num = sensor.label().split("tdie").nth(1).unwrap_or("0").trim();

            // Only adding Sensor to vec if an entry does not already exist
            if !sensors.iter().any(|(label, _)| label.contains(tdie_num)) {
                sensors.push((sensor.label(), sensor.temperature().to_f64()));
            }
        }
    }
    
    // Sorting data in Sensors Vec by tdie num
    sensors.sort_by(|(a_label, _), (b_label, _)| {
        let a_num = a_label.split("tdie").nth(1).unwrap_or("0").trim().parse::<i32>().unwrap_or(0);
        let b_num = b_label.split("tdie").nth(1).unwrap_or("0").trim().parse::<i32>().unwrap_or(0);
        a_num.cmp(&b_num)
    });

    for (label, temperature) in sensors {
        sensor_temp_string.push_str(&format!("{:<20}Temp: {:.2}°C\n", label, temperature.to_string()));

        let history = app.sensor_history.entry(label.to_string()).or_insert_with(|| VecDeque::with_capacity(100));
        if history.len() >= 100 {
            history.pop_front();
        }
        history.push_back(temperature.to_f64());
    }

    sensor_temp_string
}

fn disk_info(sys: &System) -> String {
    let mut disk_data_string = String::from("\n=> Disk Information\n");

    // Data about Disk usage 
    for disk in sys.disks(){
        disk_data_string.push_str(&format!("{}, {}, Space Left: {:.0} GB\n", 
        &disk.name().to_string_lossy(), 
        &disk.mount_point().to_string_lossy(),
        &disk.available_space().to_f64() * 9.31323E-10, 
        ));
    }
    disk_data_string
}






