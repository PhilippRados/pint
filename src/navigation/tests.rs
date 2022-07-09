#[cfg(test)]
mod tests {
    use crate::decoder;
    use crate::navigation::*;
    use std::fs::File;

    #[test]
    fn get_correct_block_size_red() {
        let mut file = File::open("tests/fixtures/piet_hello_world.png").unwrap();
        decoder::check_valid_png(&mut file);
        let rgb_img = decoder::decode_png(file);

        let block = get_block(&rgb_img, Coordinates { x: 0, y: 0 }, 5);
        let result = get_size(&block);

        assert_eq!(result, 72);
    }
    #[test]
    fn get_correct_block_size_pink() {
        let mut file = File::open("tests/fixtures/piet_hello_world.png").unwrap();
        decoder::check_valid_png(&mut file);
        let rgb_img = decoder::decode_png(file);

        let block = get_block(&rgb_img, Coordinates { x: 60, y: 0 }, 5);
        let result = get_size(&block);
        assert_eq!(result, 101);
    }
    #[test]
    fn remove_works() {
        let cord = Coordinates { x: 25, y: 50 };
        let mut cords = vec![
            Coordinates { x: 0, y: 0 },
            Coordinates { x: 25, y: 50 },
            Coordinates { x: 25, y: 60 },
            Coordinates { x: 25, y: 50 },
        ];
        let correct = vec![Coordinates { x: 0, y: 0 }, Coordinates { x: 25, y: 60 }];

        remove_all::<Coordinates>(&mut cords, &cord);
        assert_eq!(cords, correct);
    }

    #[test]
    fn furthest_dp_direction_right() {
        let block = vec![
            Coordinates { x: 0, y: 0 },
            Coordinates { x: 25, y: 40 },
            Coordinates { x: 25, y: 60 },
            Coordinates { x: 25, y: 55 },
        ];
        let dp = Direction::RIGHT;

        let result = block_dp_corners(&dp, &block);
        let expected = (Coordinates { x: 25, y: 40 }, Coordinates { x: 25, y: 60 });
        assert_eq!(result, expected);
    }

    #[test]
    fn furthest_dp_direction_up() {
        let block = vec![
            Coordinates { x: 0, y: 0 },
            Coordinates { x: 25, y: 0 },
            Coordinates { x: 40, y: 0 },
            Coordinates { x: 30, y: 0 },
            Coordinates { x: 25, y: 60 },
            Coordinates { x: 25, y: 50 },
        ];
        let dp = Direction::UP;

        let result = block_dp_corners(&dp, &block);
        let expected = (Coordinates { x: 0, y: 0 }, Coordinates { x: 40, y: 0 });
        assert_eq!(result, expected);
    }
    #[test]
    fn furthest_dp_direction_left() {
        let block = vec![
            Coordinates { x: 30, y: 10 },
            Coordinates { x: 40, y: 0 },
            Coordinates { x: 100, y: 60 },
            Coordinates { x: 200, y: 50 },
        ];
        let dp = Direction::LEFT;

        let result = block_dp_corners(&dp, &block);
        let expected = (Coordinates { x: 30, y: 10 }, Coordinates { x: 30, y: 10 });
        assert_eq!(result, expected);
    }

    #[test]
    fn next_pos_in_middle_of_image() {
        let mut file = File::open("tests/fixtures/piet_hello_world.png").unwrap();
        decoder::check_valid_png(&mut file);
        let rgb_img = decoder::decode_png(file);

        let dp = Direction::RIGHT;
        let cc = CodelChooser::RIGHT;
        let pos = Coordinates { x: 15, y: 55 };
        let codel_size = 5;

        let block = get_block(&rgb_img, pos, codel_size);
        let result = next_pos(&dp, &cc, &block, codel_size, &rgb_img);

        let expected = Coordinates { x: 40, y: 75 };
        // assert_eq!(result.unwrap(), expected);
        assert_eq!(result.unwrap(), expected);
    }
    #[test]
    fn in_range_bounds() {
        let mut file = File::open("tests/fixtures/piet_hello_world.png").unwrap();
        decoder::check_valid_png(&mut file);
        let rgb_img = decoder::decode_png(file);

        let coordinates = Coordinates { x: 150, y: 0 };
        let result = in_range(&coordinates, &rgb_img);

        assert_eq!(result, false);
    }
    #[test]
    fn navigates_test_img() {
        let mut file = File::open("tests/fixtures/piet_hello_world.png").unwrap();
        decoder::check_valid_png(&mut file);
        let rgb_img = decoder::decode_png(file);
        let mut pos = Coordinates { x: 0, y: 0 };
        let mut block: Vec<Coordinates>;
        let codel_size = 5;

        let mut result = Vec::new();
        let mut dp = Direction::RIGHT;
        let mut cc = CodelChooser::LEFT;

        let mut cc_toggled = false;
        let mut rotations = 0;
        loop {
            block = get_block(&rgb_img, pos, codel_size);
            pos = match next_pos(&dp, &cc, &block, codel_size, &rgb_img) {
                Some(new_pos) => {
                    cc_toggled = false;
                    rotations = 0;
                    new_pos
                }
                None => {
                    if rotations >= 4 {
                        break;
                    } else if cc_toggled {
                        dp = dp.next();
                        cc_toggled = false;
                        rotations += 1;
                    } else {
                        cc = cc.toggle();
                        cc_toggled = true;
                    }
                    continue;
                }
            };
            result.push(pos);
        }
        let expected = vec![
            Coordinates { x: 55, y: 0 },
            Coordinates { x: 60, y: 0 },
            Coordinates { x: 95, y: 0 },
            Coordinates { x: 100, y: 0 },
            Coordinates { x: 145, y: 55 },
            Coordinates { x: 140, y: 60 },
            Coordinates { x: 140, y: 65 },
            Coordinates { x: 140, y: 70 },
            Coordinates { x: 140, y: 75 },
            Coordinates { x: 125, y: 115 },
            Coordinates { x: 125, y: 120 },
            Coordinates { x: 125, y: 125 },
            Coordinates { x: 105, y: 140 },
            Coordinates { x: 100, y: 140 },
            Coordinates { x: 45, y: 140 },
            Coordinates { x: 40, y: 140 },
            Coordinates { x: 35, y: 140 },
            Coordinates { x: 40, y: 75 },
            Coordinates { x: 45, y: 75 },
            Coordinates { x: 50, y: 75 },
            Coordinates { x: 15, y: 65 },
            Coordinates { x: 10, y: 65 },
            Coordinates { x: 20, y: 45 },
            Coordinates { x: 20, y: 40 },
        ];

        assert_eq!(result, expected);
    }
    #[test]
    fn get_correct_colors() {
        let mut file = File::open("tests/fixtures/piet_hello_world.png").unwrap();
        decoder::check_valid_png(&mut file);
        let rgb_img = decoder::decode_png(file);
        let mut pos = Coordinates { x: 0, y: 0 };
        let codel_size = 5;

        let mut result = Vec::new();
        let mut dp = Direction::RIGHT;
        let mut cc = CodelChooser::LEFT;

        loop {
            let color = match next_color(&rgb_img, &mut pos, codel_size, &mut dp, &mut cc) {
                Some(new_color) => new_color.color,
                None => break,
            };
            result.push(color);
        }
        // let result = next_color(&rgb_img, &mut pos, codel_size, &mut dp, &mut cc)
        let expected = vec![
            RGB(192, 0, 0),
            RGB(255, 0, 255),
            RGB(192, 0, 192),
            RGB(0, 0, 255),
            RGB(0, 0, 192),
            RGB(0, 192, 0),
            RGB(192, 0, 0),
            RGB(255, 0, 255),
            RGB(192, 192, 255),
            RGB(0, 0, 255),
            RGB(0, 255, 0),
            RGB(255, 255, 192),
            RGB(255, 255, 0),
            RGB(255, 192, 192),
            RGB(255, 0, 0),
            RGB(255, 192, 255),
            RGB(0, 0, 192),
            RGB(192, 192, 255),
            RGB(0, 192, 192),
            RGB(0, 255, 0),
            RGB(0, 192, 0),
            RGB(255, 255, 0),
            RGB(192, 192, 0),
            RGB(255, 0, 0),
        ];

        assert_eq!(result, expected);
    }
    // #[test]
    // fn get_correct_colors_valentines() {
    //     let mut file = File::open("tests/fixtures/valentines.png").unwrap();
    //     decoder::check_valid_png(&mut file);
    //     let rgb_img = decoder::decode_png(file);
    //     let mut pos = Coordinates { x: 0, y: 0 };
    //     let codel_size = 1;

    //     let mut result = Vec::new();
    //     let mut dp = Direction::RIGHT;
    //     let mut cc = CodelChooser::LEFT;

    //     for i in 0..30 {
    //         let color = match next_color(&rgb_img, &mut pos, codel_size, &mut dp, &mut cc) {
    //             Some(new_color) => new_color.color,
    //             None => break,
    //         };
    //         dbg!(color);
    //         result.push(color);
    //     }
    //     // let result = next_color(&rgb_img, &mut pos, codel_size, &mut dp, &mut cc)
    //     let expected = vec![
    //         RGB(192, 0, 0),
    //         RGB(0, 0, 192),
    //         RGB(192, 255, 192),
    //         RGB(192, 192, 0),
    //         RGB(255, 255, 0),
    //         RGB(192, 192, 0),
    //         RGB(0, 0, 192),
    //         RGB(0, 255, 255),
    //         RGB(192, 0, 0),
    //         // RGB(192, 192, 255),
    //         // RGB(255, 255, 0),
    //         // RGB(192, 192, 0),
    //         // RGB(255, 255, 192),
    //     ];

    //     assert_eq!(result, expected);
    // }
}
