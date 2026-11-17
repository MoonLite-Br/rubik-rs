//! Beginner layer-by-layer course (white on the bottom, yellow on top, green in front).
//! Every case start-state is computed as "solved cube + inverse(algorithm)", so playing the
//! algorithm always ends on the cube the lesson promises. All algorithms were checked
//! against the cube model (see tests/engine.rs).

use crate::cube::{invert_alg, parse_alg, repeat_alg, Cube, Move};
use crate::i18n::Tr;

pub struct Case {
    pub name: Tr,
    pub hint: Tr,
    pub alg: &'static str,
    /// The algorithm is repeated this many times (e.g. R U R' U' x5).
    pub reps: u8,
    /// true: start from a solved cube and just show the moves (notation lesson).
    pub from_solved: bool,
}

pub struct Lesson {
    pub title: Tr,
    pub sub: Tr,
    pub intro: Tr,
    pub cases: &'static [Case],
}

impl Case {
    pub fn moves(&self) -> Vec<Move> {
        repeat_alg(&parse_alg(self.alg), self.reps as usize)
    }

    pub fn start(&self) -> Cube {
        let mut c = Cube::solved();
        if !self.from_solved {
            c.apply_all(&invert_alg(&self.moves()));
        }
        c
    }
}

const fn case(name: Tr, hint: Tr, alg: &'static str, reps: u8) -> Case {
    Case { name, hint, alg, reps, from_solved: false }
}

const fn demo(name: Tr, hint: Tr, alg: &'static str) -> Case {
    Case { name, hint, alg, reps: 1, from_solved: true }
}

const TRIGGER: &str = "R U R' U'";

pub static LESSONS: [Lesson; 8] = [
    // 0 ------------------------------------------------------------------ notation
    Lesson {
        title: Tr::new("Notation", "Ký hiệu"),
        sub: Tr::new("How to read moves", "Cách đọc thuật toán"),
        intro: Tr::new(
            "Each letter turns one face a quarter turn clockwise, as if you were looking straight at it: U (up), D (down), L (left), R (right), F (front), B (back). A prime (') means counter-clockwise and a 2 means a half turn. Lowercase x, y, z turn the whole cube.",
            "Mỗi chữ cái là xoay một mặt 90° theo chiều kim đồng hồ khi bạn nhìn thẳng vào mặt đó: U (trên), D (dưới), L (trái), R (phải), F (trước), B (sau). Dấu nháy (') là xoay ngược chiều kim đồng hồ, số 2 là xoay nửa vòng. Chữ thường x, y, z xoay cả khối.",
        ),
        cases: &[
            demo(
                Tr::new("Clockwise and back", "Thuận và ngược chiều"),
                Tr::new("R and R' cancel each other out. Step through the four moves.", "R và R' triệt tiêu nhau. Hãy bấm từng bước qua bốn nước đi."),
                TRIGGER,
            ),
            demo(
                Tr::new("Half turns", "Xoay nửa vòng"),
                Tr::new("A 2 means turn twice; the direction does not matter.", "Số 2 là xoay hai lần, không cần quan tâm chiều."),
                "R2 U2 F2",
            ),
            demo(
                Tr::new("Whole-cube turns", "Xoay cả khối"),
                Tr::new("x follows R, y follows U and z follows F.", "x xoay theo chiều R, y theo chiều U và z theo chiều F."),
                "x y z",
            ),
        ],
    },
    // 1 ------------------------------------------------------------------ white cross
    Lesson {
        title: Tr::new("White cross", "Dấu cộng trắng"),
        sub: Tr::new("Build the cross on the bottom", "Tạo dấu cộng ở mặt dưới"),
        intro: Tr::new(
            "Hold the cube with the white centre on the bottom and the green centre in front. Build a white cross on the bottom face; every white edge must also match the colour of the side centre next to it. Work it out by eye - these three cases cover most situations.",
            "Cầm khối sao cho tâm trắng ở dưới và tâm xanh lá ở trước. Hãy tạo dấu cộng trắng ở mặt dưới; mỗi cạnh trắng còn phải khớp màu với tâm của mặt bên cạnh. Bạn có thể tự quan sát và xoay - ba tình huống dưới đây xử lý hầu hết các trường hợp.",
        ),
        cases: &[
            case(
                Tr::new("White on top", "Trắng ở mặt trên"),
                Tr::new(
                    "The white edge is on top and its other colour is in front, matching the front centre. Turn the top layer until they line up, then F2.",
                    "Cạnh trắng nằm ở mặt trên, màu còn lại ở mặt trước và khớp với tâm trước. Xoay tầng trên cho thẳng hàng rồi F2.",
                ),
                "F2",
                1,
            ),
            case(
                Tr::new("White on the side", "Trắng ở mặt bên"),
                Tr::new(
                    "The white sticker faces front and the other colour is on top. Take it out and bring it back the right way round: U' R' F R.",
                    "Mặt trắng hướng ra phía trước, màu còn lại ở trên. Đưa ra rồi đưa vào lại đúng chiều: U' R' F R.",
                ),
                "U' R' F R",
                1,
            ),
            case(
                Tr::new("White in the middle layer", "Trắng ở tầng giữa"),
                Tr::new(
                    "The white edge is on the front-right of the middle layer, white facing right. One F moves it down into place.",
                    "Cạnh trắng ở tầng giữa, phía trước-phải, mặt trắng hướng sang phải. Một nước F là đưa nó xuống đúng chỗ.",
                ),
                "F",
                1,
            ),
        ],
    },
    // 2 ------------------------------------------------------------------ white corners
    Lesson {
        title: Tr::new("White corners", "Góc trắng"),
        sub: Tr::new("Finish the first layer", "Hoàn thành tầng đầu tiên"),
        intro: Tr::new(
            "Find a white corner in the top layer and bring it above its slot - the front-right corner. Repeat R U R' U' until the white sticker points down and the corner matches the side centres. Ignore the middle layer for now.",
            "Tìm góc có màu trắng ở tầng trên và đưa nó lên ngay phía trên vị trí cần đặt - góc trước-phải. Lặp lại R U R' U' cho đến khi mặt trắng hướng xuống và góc khớp với màu các tâm bên cạnh. Tạm thời chưa cần lo tầng giữa.",
        ),
        cases: &[
            case(
                Tr::new("White facing right", "Trắng hướng sang phải"),
                Tr::new("One trigger is enough.", "Chỉ cần một lần."),
                TRIGGER,
                1,
            ),
            case(
                Tr::new("White facing up", "Trắng hướng lên trên"),
                Tr::new("Repeat the trigger three times.", "Lặp lại ba lần."),
                TRIGGER,
                3,
            ),
            case(
                Tr::new("White facing front", "Trắng hướng ra trước"),
                Tr::new("Repeat the trigger five times.", "Lặp lại năm lần."),
                TRIGGER,
                5,
            ),
        ],
    },
    // 3 ------------------------------------------------------------------ middle layer
    Lesson {
        title: Tr::new("Middle layer", "Tầng giữa"),
        sub: Tr::new("Second-layer edges", "Cạnh của tầng thứ hai"),
        intro: Tr::new(
            "With white still on the bottom, look for a top-layer edge with no yellow on it. Turn the top layer until its front colour matches the front centre, then send it to the right or to the left.",
            "Khi mặt trắng vẫn ở dưới, tìm một cạnh ở tầng trên không có màu vàng. Xoay tầng trên cho màu ở mặt trước khớp với tâm trước, rồi đưa nó sang phải hoặc sang trái.",
        ),
        cases: &[
            case(
                Tr::new("Insert to the right", "Đưa sang phải"),
                Tr::new("The edge drops into the front-right slot.", "Cạnh rơi vào vị trí trước-phải."),
                "U R U' R' U' F' U F",
                1,
            ),
            case(
                Tr::new("Insert to the left", "Đưa sang trái"),
                Tr::new("The edge drops into the front-left slot.", "Cạnh rơi vào vị trí trước-trái."),
                "U' L' U L U F U' F'",
                1,
            ),
        ],
    },
    // 4 ------------------------------------------------------------------ yellow cross
    Lesson {
        title: Tr::new("Yellow cross", "Dấu cộng vàng"),
        sub: Tr::new("Orient the last layer - 1", "Định hướng tầng cuối - 1"),
        intro: Tr::new(
            "Look at the yellow face. You will see a dot, an L-shape or a line. Run the matching algorithm until a yellow cross appears. If only the dot is visible, run either algorithm once and look again.",
            "Nhìn vào mặt vàng: bạn sẽ thấy một chấm, chữ L hoặc một đường thẳng. Chạy thuật toán tương ứng cho đến khi xuất hiện dấu cộng vàng. Nếu chỉ thấy chấm, hãy chạy một trong hai thuật toán một lần rồi quan sát lại.",
        ),
        cases: &[
            case(
                Tr::new("Line", "Đường thẳng"),
                Tr::new("Hold the line horizontally.", "Cầm sao cho đường thẳng nằm ngang."),
                "F R U R' U' F'",
                1,
            ),
            case(
                Tr::new("L-shape", "Chữ L"),
                Tr::new("Hold the L in the back-left corner.", "Cầm sao cho chữ L ở góc sau-trái."),
                "F U R U' R' F'",
                1,
            ),
        ],
    },
    // 5 ------------------------------------------------------------------ yellow face
    Lesson {
        title: Tr::new("Yellow face", "Mặt vàng"),
        sub: Tr::new("Orient the last layer - 2", "Định hướng tầng cuối - 2"),
        intro: Tr::new(
            "Make the whole top face yellow. With exactly one yellow corner showing, hold it as the hint says and run the matching algorithm. If you see none or two, run Sune once from any angle and look again.",
            "Làm cho cả mặt trên có màu vàng. Khi chỉ thấy đúng một góc vàng, hãy cầm theo gợi ý và chạy thuật toán tương ứng. Nếu thấy không góc nào hoặc hai góc, chạy Sune một lần từ góc nhìn bất kỳ rồi quan sát lại.",
        ),
        cases: &[
            case(
                Tr::new("Sune", "Sune"),
                Tr::new("One yellow corner on top at the front-left.", "Một góc vàng hướng lên ở trước-trái."),
                "R U R' U R U2 R'",
                1,
            ),
            case(
                Tr::new("Anti-Sune", "Anti-Sune"),
                Tr::new("One yellow corner on top at the back-right.", "Một góc vàng hướng lên ở sau-phải."),
                "R U2 R' U' R U' R'",
                1,
            ),
        ],
    },
    // 6 ------------------------------------------------------------------ corner PLL
    Lesson {
        title: Tr::new("Place the corners", "Đặt các góc"),
        sub: Tr::new("Permute the last layer - 1", "Hoán vị tầng cuối - 1"),
        intro: Tr::new(
            "The top is yellow; now fix the positions. Find a corner that is already right - its side colours match the neighbouring centres - and hold it at the front-left. If none is right, run the algorithm once and look again.",
            "Mặt trên đã vàng; giờ chỉnh vị trí. Tìm góc đã đúng chỗ - màu hai bên khớp với các tâm kế bên - và cầm nó ở trước-trái. Nếu chưa có góc nào đúng, chạy thuật toán một lần rồi quan sát lại.",
        ),
        cases: &[case(
            Tr::new("Corner cycle", "Hoán đổi ba góc"),
            Tr::new("The three other corners rotate; the front-left one stays.", "Ba góc còn lại xoay vòng; góc trước-trái giữ nguyên."),
            "R' F R' B2 R F' R' B2 R2",
            1,
        )],
    },
    // 7 ------------------------------------------------------------------ edge PLL
    Lesson {
        title: Tr::new("Place the edges", "Đặt các cạnh"),
        sub: Tr::new("Permute the last layer - 2", "Hoán vị tầng cuối - 2"),
        intro: Tr::new(
            "Last step. Hold the one edge that is already right at the back and cycle the other three. If the edges still do not match, try the other version.",
            "Bước cuối. Cầm cạnh đã đúng chỗ ở phía sau và hoán đổi vòng ba cạnh còn lại. Nếu các cạnh vẫn chưa khớp, thử phiên bản còn lại.",
        ),
        cases: &[
            case(
                Tr::new("Edge cycle A", "Hoán đổi cạnh - A"),
                Tr::new("Keep the solved edge at the back.", "Giữ cạnh đã đúng ở phía sau."),
                "R U' R U R U R U' R' U' R2",
                1,
            ),
            case(
                Tr::new("Edge cycle B", "Hoán đổi cạnh - B"),
                Tr::new("Keep the solved edge at the back.", "Giữ cạnh đã đúng ở phía sau."),
                "R2 U R U R' U' R' U' R' U R'",
                1,
            ),
        ],
    },
];
