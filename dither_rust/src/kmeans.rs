use core::num;
use std::collections::HashSet;
use std::hash::Hash;

use image::{DynamicImage, GenericImage, GenericImageView, Pixel, Rgb};
use lab::Lab;

use rand::distributions::WeightedIndex;
use rand::prelude::*;
use rand::Rng;

pub struct Cluster {
    pub average_pixel: Lab,
    pub members: Vec<Lab>,
    pub score: f32,
}

impl std::fmt::Debug for Cluster {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Cluster")
            .field("average_pixel", &self.average_pixel)
            .field("average_error", &self.score)
            .field("members_length", &self.members.len())
            .finish()
    }
}

pub fn average_lab(pixels: &[Lab]) -> Lab {
    let sum = pixels
        .iter()
        .copied()
        .reduce(|accum, pixel| Lab {
            l: accum.l + pixel.l,
            a: accum.a + pixel.a,
            b: accum.b + pixel.b,
        })
        .expect("Should always have a sum");

    let n = pixels.len() as f32;
    Lab {
        l: sum.l / n,
        a: sum.a / n,
        b: sum.b / n,
    }
}

fn lab_equal(c1: &Lab, c2: &Lab) -> bool {
    let epsilon = 1e-5;
    f32_equal(c1.l, c2.l) && f32_equal(c1.a, c2.a) && f32_equal(c1.b, c2.b)
}

fn f32_equal(x: f32, y: f32) -> bool {
    let epsilon = 1e-5;
    f32::abs(x - y) < epsilon
}

fn unique_colours(colours: &[Lab]) -> Vec<Lab> {
    let mut unique = Vec::new();

    colours.iter().for_each(|c| {
        let mut clash = false;
        for existing in &unique {
            if lab_equal(c, existing) {
                clash = true;
                break;
            }
        }
        if !clash {
            unique.push(*c);
        }
    });

    unique
}

//fn improve_clusters(img: &[Lab], clusters: &[Cluster])

pub fn cluster(img: &DynamicImage, num_clusters: u32, num_iterations: u32) -> Vec<Cluster> {
    let lab_pixels: Vec<Lab> = img
        .pixels()
        .map(|(_, _, pixel)| {
            let rgb = pixel.to_rgb();
            let channels = rgb.channels();
            Lab::from_rgb(&[channels[0], channels[1], channels[2]])
        })
        .collect();

    let mut rng = rand::thread_rng();

    let unique_pixels = unique_colours(&lab_pixels);
    let mut clusters = Vec::new();
    for i in 0..num_clusters {
        if i == 0 {
            let idx = rng.gen_range(0..unique_pixels.len());
            clusters.push(Cluster {
                average_pixel: unique_pixels[idx],
                members: Vec::new(),
                score: 0.0,
            });
            continue;
        }

        let weights: Vec<f32> = unique_pixels
            .iter()
            .map(|pixel| {
                clusters
                    .iter()
                    .map(|cluster| pixel.squared_distance(&cluster.average_pixel))
                    .reduce(f32::min)
                    .expect("Should be a min")
            })
            .collect();

        let dist = WeightedIndex::new(&weights).expect("Should be able to create a distribution");
        clusters.push(Cluster {
            average_pixel: unique_pixels[dist.sample(&mut rng)],
            members: Vec::new(),
            score: 0.0,
        });
    }

    //let mut clusters: Vec<Cluster> = (0..num_clusters)
    //    .map(|_| {
    //        let idx = rng.gen_range(0..lab_pixels.len());
    //        Cluster {
    //            average_pixel: lab_pixels[idx],
    //            members: Vec::new(),
    //            score: 0.0,
    //        }
    //    })
    //    .collect();

    //let mut clusters: Vec<Cluster> = lab_pixels
    //    .iter()
    //    .take(num_clusters as usize)
    //    .map(|(pixel)| Cluster {
    //        average_pixel: pixel.clone(),
    //        members: vec![pixel.clone()],
    //    })
    //    .collect();

    let mut converged = false;
    let mut iter = 0;
    while !converged {
        println!("Iteration {}", iter);
        clusters
            .iter_mut()
            .for_each(|cluster| cluster.members.clear());

        let prev_scores: Vec<f32> = clusters.iter().map(|cluster| cluster.score).collect();

        lab_pixels.iter().for_each(|pixel| {
            let (best_cluster_idx, _) = clusters
                .iter()
                .enumerate()
                .map(|(idx, cluster)| (idx, pixel.squared_distance(&cluster.average_pixel)))
                .min_by(|(_, s1), (_, s2)| s1.partial_cmp(s2).expect("should compare"))
                .expect("There should always be a best cluster");

            clusters[best_cluster_idx].members.push(pixel.clone());
            //clusters[cluster_idx].average_pixel = average_pixel(&clusters[cluster_idx].members);
        });

        clusters.iter_mut().for_each(|cluster| {
            cluster.average_pixel = average_lab(&cluster.members);
            cluster.score = cluster.members.iter().fold(0.0, |score, pixel| {
                score + pixel.squared_distance(&cluster.average_pixel)
            }) / cluster.members.len() as f32;
        });

        //clusters
        //    .iter()
        //    .for_each(|cluster| println!("{:?}", cluster));
        //println!("\n");

        converged = clusters
            .iter()
            .map(|cluster| cluster.score)
            .zip(prev_scores)
            .all(|(s1, s2)| f32_equal(s1, s2));

        iter += 1;
    }

    clusters
}
