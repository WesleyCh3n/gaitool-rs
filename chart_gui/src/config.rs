use Position::*;
use Variable::*;

#[derive(Debug, Eq, Hash, PartialEq, Clone)]
pub enum Position {
    L,
    T,
    C,
    ScapularLT,
    ScapularRT,
    SC,
    HipLT,
    HipRT,
    KneeLT,
    KneeRT,
    ShoulderLT,
    ShoulderRT,
    PSISLT,
    PSISRT,
    FootLT,
    FootRT,
}

impl Position {
    pub fn iterator() -> std::slice::Iter<'static, Position> {
        static POSITION: [Position; 16] = [
            L, T, C, ScapularLT, ScapularRT, SC, HipLT, HipRT, KneeLT, KneeRT,
            ShoulderLT, ShoulderRT, PSISLT, PSISRT, FootLT, FootRT,
        ];
        POSITION.iter()
    }
    pub fn to_str(p: &Position) -> &'static str {
        match p {
            L => "L",
            T => "T",
            C => "C",
            ScapularLT => "Scapular LT",
            ScapularRT => "Scapular RT",
            SC => "SC",
            HipLT => "HIP LT",
            HipRT => "HIP RT",
            KneeLT => "Knee LT",
            KneeRT => "Knee RT",
            ShoulderLT => "Shoulder LT",
            ShoulderRT => "Shoulder RT",
            PSISLT => "PSIS LT",
            PSISRT => "PSIS RT",
            FootLT => "Foot LT",
            FootRT => "Foot RT",
        }
    }
}

#[derive(Debug, Eq, Hash, PartialEq, Clone)]
pub enum Variable {
    AccelX,
    AccelY,
    AccelZ,
    Course,
    Pitch,
    Roll,
    MagnetX,
    MagnetY,
    MagnetZ,
}

impl Variable {
    pub fn iterator() -> std::slice::Iter<'static, Variable> {
        static VARIABLE: [Variable; 9] = [
            AccelX, AccelY, AccelZ, Course, Pitch, Roll, MagnetX, MagnetY,
            MagnetZ,
        ];
        VARIABLE.iter()
    }
    pub fn to_name_string(v: &Variable, p: &Position) -> String {
        let name = Position::to_str(p);
        let p_vec: Vec<&str> = name.split(" ").collect();
        let first_arg = p_vec[0];
        let sec_arg = if p_vec.len() == 2 {
            format!("{} ", p_vec[1])
        } else {
            "".to_owned()
        };

        // HACK: stupid inconsistent naming rule
        let (course, pitch, roll) = if p == &Position::L {
            (
                format!(
                    "Noraxon MyoMotion-Segments-{}-Course (deg)",
                    Position::to_str(&Position::L)
                ),
                format!(
                    "Noraxon MyoMotion-Segments-{}-Pitch (deg)",
                    Position::to_str(&Position::L)
                ),
                format!(
                    "Noraxon MyoMotion-Segments-{}-Roll (deg)",
                    Position::to_str(&Position::L)
                ),
            )
        } else {
            (
                format!("{} course {}(deg)", first_arg, sec_arg),
                format!("{} pitch {}(deg)", first_arg, sec_arg),
                format!("{} roll {}(deg)", first_arg, sec_arg),
            )
        };

        match v {
            AccelX => format!("{} Accel Sensor X {}(mG)", first_arg, sec_arg),
            AccelY => format!("{} Accel Sensor Y {}(mG)", first_arg, sec_arg),
            AccelZ => format!("{} Accel Sensor Z {}(mG)", first_arg, sec_arg),
            Course => course,
            Pitch => pitch,
            Roll => roll,
            MagnetX => {
                format!(
                    "Noraxon MyoMotion-Segments-{}-Magnetometer-x (mGauss)",
                    name
                )
            }
            MagnetY => {
                format!(
                    "Noraxon MyoMotion-Segments-{}-Magnetometer-y (mGauss)",
                    name
                )
            }
            MagnetZ => format!(
                "Noraxon MyoMotion-Segments-{}-Magnetometer-z (mGauss)",
                name
            ),
        }
    }
}
