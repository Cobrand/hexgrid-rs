//            +y
//         ^
//        | | +x
//         V
//

use std::ops::{Sub,Add,Neg};
use std::convert::From;

pub enum Orientation {
    NE,
    E,
    SE,
    SW,
    W,
    NW
}

#[derive(Copy,Clone,Debug,PartialEq,Eq)]
pub struct Position {
    pub x : i32,
    pub y : i32
}

impl Position {
    pub fn new(x:i32,y:i32) -> Position {
        Position {x:x,y:y}
    }

    pub fn from_orientation(orientation:Orientation,length:i32) -> Position {
        match orientation {
            Orientation::NE => Position::new(0,length),
            Orientation::E => Position::new(length,0),
            Orientation::SE => Position::new(length,-length),
            _ => Position::new(0,0)
        }
    }

    pub fn get_z(&self) -> i32 {
        -self.y - self.x
    }
}

impl Sub for Position {
    type Output = Self;
    fn sub(self, rhs:Self ) -> Self {
        Position::new(self.x - rhs.x , self.y - rhs.y)
    }
}



impl Add for Position {
    type Output = Self;
    fn add(self, rhs:Self ) -> Self {
        Position::new(self.x + rhs.x , self.y + rhs.y)
    }
}

impl Add<(i32,i32)> for Position {
    type Output = Self;
    fn add(self, rhs:(i32,i32) ) -> Self {
        Position::new(self.x + rhs.0 , self.y + rhs.1)
    }
}

impl Add<Position> for (i32,i32 ){
    type Output = Position;
    fn add(self, rhs:Position ) -> Position {
        rhs + self
    }
}

impl Neg for Position {
    type Output = Position ;
    fn neg(self) -> Position {
        Position::new(-self.x,-self.y)
    }
}

impl Sub<(i32,i32)> for Position {
    type Output = Self;
    fn sub(self, rhs:(i32,i32) ) -> Position {
        Position::new(self.x - rhs.0 , self.y - rhs.1)
    }
}

impl Sub<Position> for (i32,i32 ){
    type Output = Position;
    fn sub(self, rhs:Position ) -> Position {
        -(rhs - self)
    }
}

impl From<(i32,i32)> for Position {
    fn from(tuple:(i32,i32)) -> Position {
        Position::new(tuple.0,tuple.1)
    }
}

#[cfg(test)]
mod tests {
    use super::* ;

    #[test]
    fn get_z(){
        assert_eq!(Position::new(1,0).get_z(),-1);
        assert_eq!(Position::new(0,0).get_z(),0);
        assert_eq!(Position::new(5,-2).get_z(),-3);
    }

    #[test]
    fn eq_position(){
        assert_eq!(Position::new(5,2),Position::new(5,2));
    }

    #[test]
    fn sub_position(){
        let position : Position = Position::new(5,2);
        assert_eq!(position - Position::new(1,-1),Position::new(4,3));
    }

    #[test]
    fn sub_tuple(){
        let position : Position = Position::new(5,2);
        assert_eq!(position - (5i32,-2i32),Position::new(0,4));
    }

    #[test]
    fn add_position(){
        let position : Position = Position::new(5,2);
        assert_eq!(position + Position::new(1,-1),Position::new(6,1));
    }

    #[test]
    fn add_tuple(){
        let position : Position = Position::new(5,2);
        let tuple : (i32,i32) = (1,-1);
        assert_eq!(tuple + position,Position::new(6,1));
        assert_eq!(position + tuple,Position::new(6,1));
    }
}