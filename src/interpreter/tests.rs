#[cfg(test)]
mod tests {
    use crate::interpreter::*;
    use crate::types::*;

    #[test]
    fn gets_correct_color_index() {
        let color = RGB(0, 255, 0);
        let result = get_color_index(color).unwrap();
        let expected = Coordinates { x: 2, y: 1 };

        assert_eq!(result, expected);
    }

    #[test]
    fn gets_correct_color_diff_1() {
        let prev = RGB(0, 255, 0);
        let current = RGB(255, 192, 192);
        let result = calculate_color_diff(prev, current);
        let expected = Coordinates { x: 4, y: 2 };

        assert_eq!(result, expected);
    }

    #[test]
    fn gets_correct_color_diff_2() {
        let prev = RGB(192, 255, 192);
        let current = RGB(192, 255, 255);
        let result = calculate_color_diff(prev, current);
        let expected = Coordinates { x: 1, y: 0 };

        assert_eq!(result, expected);
    }
    #[test]
    fn gets_correct_color_diff_3() {
        let prev = RGB(192, 0, 192);
        let current = RGB(255, 0, 255);
        let result = calculate_color_diff(prev, current);
        let expected = Coordinates { x: 0, y: 2 };

        assert_eq!(result, expected);
    }
    #[test]
    fn roll_test1() {
        let mut stack = vec![12, 3, 102, 33, 7, 4, 2];
        let mut dp = Direction::UP;
        let mut cc = CodelChooser::LEFT;

        roll(3, &mut stack, &mut cc, &mut dp);
        assert_eq!(stack, [12, 33, 7, 3, 102]);
    }
    #[test]
    fn roll_test2() {
        let mut stack = vec![1, 2, 3, 3, 1];
        let mut dp = Direction::UP;
        let mut cc = CodelChooser::LEFT;

        roll(3, &mut stack, &mut cc, &mut dp);
        assert_eq!(stack, [3, 1, 2]);
    }
    #[test]
    fn switch_test() {
        let mut stack = vec![1, 2, 3, 3, 1];
        let mut dp = Direction::UP;
        let mut cc = CodelChooser::LEFT;

        switch(3, &mut stack, &mut cc, &mut dp);
        assert_eq!(stack, [1, 2, 3, 3]);
        assert_eq!(cc, CodelChooser::RIGHT);
    }
}
