use bevy::{prelude::*, window::WindowResolution};
use bevy_pixels::prelude::*;
use robotics_lib::world::{worldgenerator::Generator, tile::Tile, tile::{TileType, Content}, environmental_conditions::{EnvironmentalConditions, WeatherType}};
use noise::{Add, Blend, Fbm, NoiseFn, Perlin, RidgedMulti, Worley};
use noise::core::worley;

pub struct WorldGenerator {
    seed: u32,
    world_size: usize,
}

#[derive(Resource)]
struct WorldMatrixResource {
    matrix: Vec<Vec<Tile>>
}

impl WorldGenerator {
    pub fn new(seed: u32, world_size: usize) -> Self {
        Self { seed, world_size }
    }

    fn generate_weather(&self) -> EnvironmentalConditions {
        EnvironmentalConditions::new(
            &[WeatherType::Sunny, WeatherType::Rainy],
            1,
            0
        ).unwrap()
    }

    fn generate_altitude(&self) -> Vec<Vec<f64>> {
        let perlin = Perlin::new(self.seed);
        let mut elevation_map = vec![vec![0.0; self.world_size]; self.world_size];
        
        for x in 0..self.world_size {
            for y in 0..self.world_size {
                let scale = 1.0/50.0;
                elevation_map[x][y] = perlin.get([x as f64 * scale, y as f64 * scale]);
            }
        }

        return elevation_map;
    }

    fn generate_biomes(&self, elevation_map: Vec<Vec<f64>>) -> Vec<Vec<Tile>> {
        let deep_water_tile = Tile { tile_type: TileType::DeepWater, content: Content::None, elevation: 0};
        let shallow_water_tile = Tile { tile_type: TileType::ShallowWater, content: Content::None, elevation: 0};
        let grass_tile = Tile { tile_type: TileType::Grass, content: Content::None, elevation: 0};
        let sand_tile = Tile { tile_type: TileType::Sand, content: Content::None, elevation: 0};
        let lava_tile = Tile { tile_type: TileType::Lava, content: Content::None, elevation: 0};
        let hill_tile = Tile { tile_type: TileType::Hill, content: Content::None, elevation: 0};
        let mountain_tile = Tile { tile_type: TileType::Mountain, content: Content::None, elevation: 0};
        let snow_tile = Tile { tile_type: TileType::Snow, content: Content::None, elevation: 0};

        let mut world = vec![vec![deep_water_tile.clone(); self.world_size]; self.world_size];
        let mut temperature_map: Vec<Vec<f64>> = vec![vec![0.0; self.world_size]; self.world_size];

        let perlin = Perlin::new(self.seed + 42);
        for x in 0..self.world_size {
            for y in 0..self.world_size {
                let scale = 1.0/50.0;
                temperature_map[x][y] = (
                    1.00 * perlin.get( [scale * 1.0 * x as f64, scale *  1.0 * y as f64])
                    + 0.75 * perlin.get( [scale * 2.0 * x as f64, scale *  2.0 * y as f64])
                    + 0.33 * perlin.get( [scale * 4.0 * x as f64, scale *  4.0 * y as f64])
                    + 0.33 * perlin.get( [scale * 8.0 * x as f64, scale *  8.0 * y as f64])
                    + 0.33 * perlin.get([scale * 16.0 * x as f64, scale * 16.0 * y as f64])
                    + 0.50 * perlin.get([scale * 32.0 * x as f64, scale * 32.0 * y as f64])
                ) / (1.00 + 0.75 + 0.33 + 0.33 + 0.33 + 0.50);
                //println!("{}", temperature_map[x][y]);
            }
        }

        let get_biome = |x: usize, y: usize| -> Tile {
            println!("{}", temperature_map[x][y]);

            match temperature_map[x][y] {
                //plains
                v if v > -0.5 && v <= 0.0 => grass_tile.clone(),
                //desert
                v if v >  0.0 && v < 0.5 => sand_tile.clone(),
                //forest
                _ => lava_tile.clone()
            }
        };

        for x in 0..self.world_size {
            for y in 0..self.world_size {
                world[x][y] = match elevation_map[x][y] {
                    h if h < -0.75 => deep_water_tile.clone(),
                    h if h < -0.50 => shallow_water_tile.clone(),
                    h if h <  0.50 => get_biome(x, y),
                    h if h <  0.65 => hill_tile.clone(),
                    h if h <  0.75 => mountain_tile.clone(),
                    h if h <= 1.00 => snow_tile.clone(),
                    _ => deep_water_tile.clone()
                };
            }
        }

        return world;
    }

    fn generate_spawnpoint(&self) -> (usize, usize) {
        (0, 0)
    }

    fn color_tile(tile: &Tile) -> [u8; 4] {
        return match tile.tile_type {
            TileType::DeepWater => [0, 0, 127, 255],
            TileType::ShallowWater => [0, 0, 255, 255],
            TileType::Grass => [0, 255, 0, 255],
            TileType::Sand => [255, 255, 0, 255],
            TileType::Lava => [255, 0, 0, 255],
            TileType::Hill => [0, 127, 0, 255],
            TileType::Mountain => [153, 102, 51, 255],
            TileType::Snow => [255, 255, 255, 255],
            _ => [0, 0, 0, 255]
        }
    }

    fn draw_window(mut wrapper_query: Query<&mut PixelsWrapper>, world: Res<WorldMatrixResource>) {
        //Bevy pixels stuff
        let Ok(mut wrapper) = wrapper_query.get_single_mut() else { return };
        wrapper.pixels.resize_buffer(world.matrix.len() as u32, world.matrix.len() as u32).unwrap();
        let frame = wrapper.pixels.frame_mut();

        assert_eq!(world.matrix.len() * world.matrix.len() * 4, frame.len());

        for i in 0..(frame.len() / 4) {
            let color = Self::color_tile(&world.matrix[i % world.matrix.len()][i / world.matrix.len()]);
            frame[i*4..i*4+4].copy_from_slice(&color);
        }
    }

    pub fn visualize(world: Vec<Vec<Tile>>, resolution : usize) {
        let mut window_resolution = WindowResolution::new(resolution as f32, resolution as f32);
        window_resolution.set_scale_factor_override(Some(1.0)); // on smaller screens bevy sometimes sets the scale factor

        let window_plugin = WindowPlugin {
            primary_window: Some(Window {
                title: "MIDGARD".into(),
                resolution: window_resolution,
                resizable: false,
                ..default()
            }),
            ..default()
        };


        App::new()
            .add_plugins((DefaultPlugins.set(window_plugin), PixelsPlugin::default()))
            .add_systems(Update, bevy::window::close_on_esc)
            .add_systems(Draw, Self::draw_window)
            .insert_resource(WorldMatrixResource{ matrix: world })
            .run();
    }
}

impl Generator for WorldGenerator {
    fn gen(&mut self) -> (Vec<Vec<Tile>>, (usize, usize), EnvironmentalConditions, f32) {    
        let altitude_map = self.generate_altitude();
        let world = self.generate_biomes(altitude_map);

        let weather = self.generate_weather();
        let spawnpoint = self.generate_spawnpoint();
        let score = 100.0;

        (world, spawnpoint, weather, score)
    }
}
