use error::{Error,Reason,Result};
use pos::Position;
use std::vec::Vec ;
use std::iter::{Iterator,Zip};
use std::slice::{Iter,IterMut};
use std::mem::replace;

pub trait PositionAccessor {
    fn set_position(&mut self,new_position:Position);
    fn get_position(&self) -> Position ;
}

pub trait AllowContent {
    fn is_content_allowed(&self) -> bool ;
}

pub struct MapIter<I>{
    iter:I,
    current_index:usize,
    length: (i32,i32),
    offset: Position
}

pub enum PositionStatus {
    /// Bg has position allowed and no element is at this Position
    Empty,
    /// Bg has position allowed but an element is already at this position
    Busy,
    /// Bg does not allow content at this Position
    Forbidden
}

impl<I> Iterator for MapIter<I> where I : Iterator {
    type Item = (Position,<I as Iterator>::Item) ;
    #[inline]
    fn next(&mut self) -> Option<(Position, <I as Iterator>::Item)> {
        let position = index_to_pos(self.current_index, self.length, self.offset);
        match position {
            Ok(position) => self.iter.next().map(|a| {
                let ret = (position, a);
                self.current_index += 1;
                ret
            }),
            Err(err) if err == Error::new(Reason::OutOfRange) => None,
            Err(_) => unreachable!()
        }
    }
}

impl<I> MapIter<I> where I : Iterator {
    pub fn new(iter:I,length: (i32,i32),offset: Position) -> MapIter<I> {
        MapIter {
            current_index:0,
            iter:iter,
            length:length,
            offset:offset
        }
    }
}

pub struct Map<T : PositionAccessor,Bg : Default + AllowContent > {
    contents_slice : Box<[Option<T>]>,
    bg_slice : Box<[Bg]>,
    length: (i32,i32),
    offset: Position
}

impl<T,Bg> Map<T,Bg> where T : PositionAccessor, Bg : Default + AllowContent {
    pub fn new(length:(i32,i32),offset:Position) -> Result<Map<T,Bg>> {
        if length.0 <= 0 || length.1 <= 0 {
            Err(Error::new(Reason::NegativeMapLength))
        } else {
            let total_len : usize = length.0 as usize * length.1 as usize ;
            let mut contents_vec : Vec<Option<T>> = Vec::with_capacity(total_len);
            let mut bg_vec : Vec<Bg> = Vec::with_capacity(total_len);
            for _i in 0 .. total_len {
                contents_vec.push(None);
                bg_vec.push(Bg::default());
            };
            Ok(Map::<T,Bg> {
                contents_slice:contents_vec.into_boxed_slice(),
                bg_slice:bg_vec.into_boxed_slice(),
                length:length,
                offset:offset
            })
        }
    }

    pub fn position_status(&self,position:Position) -> Result<PositionStatus> {
        let index = try!(self.pos_to_index(position));
        let (contents, bg) = self.get_unchecked(index);
        let result = match *contents {
            Some(_) => PositionStatus::Busy,
            None if bg.is_content_allowed() => PositionStatus::Empty,
            None if !bg.is_content_allowed() => PositionStatus::Forbidden,
            _ => unreachable!()
        };
        Ok(result)
    }

    pub fn from_iter<I>(iter:I,length:(i32,i32),offset:Position) -> Result<Map<T,Bg>> where I : IntoIterator<Item=(Position,(T,Bg))> {
        let mut map = try!(Self::new(length,offset));
        for (pos,(content,bg)) in iter {
            try!(map.create_content(pos,content));
            let mut map_bg = map.get_bg_mut(pos).unwrap();
            *map_bg = bg ;
        }
        Ok(map)
    }

    fn pos_to_index(&self,pos:Position) -> Result<usize> {
        debug_assert!(self.length.0 > 0 && self.length.1 > 0);
        let tmp_pos = pos - Position::from(self.offset) ;
        if tmp_pos.x < 0 || tmp_pos.x >= self.length.0
        || tmp_pos.y < 0 || tmp_pos.y >= self.length.1 {
            Err(Error::new(Reason::OutOfRange))
        } else {
            Ok((tmp_pos.x + self.length.0 * tmp_pos.y) as usize)
        }
    }

    #[allow(dead_code)]
    fn index_to_pos(&self,index:usize) -> Result<Position> {
        index_to_pos(index, self.length, self.offset)
    }

    #[inline]
    fn get_unchecked(&self,index:usize) -> (&Option<T>,&Bg) {
        unsafe {
            (self.contents_slice.get_unchecked(index),self.bg_slice.get_unchecked(index))
        }
    }

    #[inline]
    fn get_unchecked_mut(&mut self,index:usize) -> (&mut Option<T>,&mut Bg) {
        unsafe {
            (self.contents_slice.get_unchecked_mut(index),self.bg_slice.get_unchecked_mut(index))
        }
    }

    /// # Errors
    ///
    /// * `OutOfRange` if position is not valid
    pub fn get_mut(&mut self,pos:Position) -> Result<(&mut Option<T>,&mut Bg)> {
        let index = try!(self.pos_to_index(pos));
        Ok((&mut self.contents_slice[index],&mut self.bg_slice[index]))
    }

    /// # Errors
    ///
    /// * `OutOfRange` if position is not valid
    pub fn get(&self,pos:Position) -> Result<(&Option<T>,&Bg)> {
        let index = try!(self.pos_to_index(pos));
        Ok((&self.contents_slice[index],&self.bg_slice[index]))
    }

    /// # Errors
    ///
    /// * `OutOfRange` if position is not valid
    pub fn get_contents(&self,pos:Position) -> Result<&Option<T>> {
        let index = try!(self.pos_to_index(pos));
        Ok(&self.contents_slice[index])
    }

    /// # Errors
    ///
    /// * `OutOfRange` if position is not valid
    pub fn get_contents_mut(&mut self,pos:Position) -> Result<&mut Option<T>> {
        let index = try!(self.pos_to_index(pos));
        Ok(&mut self.contents_slice[index])
    }

    /// # Errors
    ///
    /// * `OutOfRange` if position is not valid
    pub fn get_bg(&self,pos:Position) -> Result<&Bg> {
        let index = try!(self.pos_to_index(pos));
        Ok(&self.bg_slice[index])
    }

    /// # Errors
    ///
    /// * `OutOfRange` if position is not valid
    pub fn get_bg_mut(&mut self,pos:Position) -> Result<&mut Bg>{
        let index = try!(self.pos_to_index(pos));
        Ok(&mut self.bg_slice[index])
    }

    /// Replace a `Position` with a new content.
    ///
    /// # Errors
    ///
    /// * `OutOfRange` if position is not valid
    /// * `MissingTarget` if Position has no content (`None`)
    pub fn replace_content(&mut self,position:Position,mut new_content:T) -> Result<T> {
        let index = try!(self.pos_to_index(position));
        if self.contents_slice[index].is_some() {
            new_content.set_position(position);
            let replaced = replace(&mut self.contents_slice[index],Some(new_content));
            Ok(replaced.expect("Unexpected None"))
        } else {
            Err(Error::new(Reason::MissingTarget))
        }
    }

    /// Extract a content at `Position` and replace it with `None`.
    ///
    /// # Errors
    ///
    /// * `OutOfRange` if position is not valid
    /// * `MissingTarget` if Position has no content (`None`)
    pub fn extract_content(&mut self,position:Position) -> Result<T> {
        let index = try!(self.pos_to_index(position));
        if self.contents_slice[index].is_some() {
            let ref mut content : Option<T> = self.contents_slice[index];
            Ok(replace(content,None).unwrap())
        } else {
            Err(Error::new(Reason::MissingTarget))
        }
    }

    /// Extract a content at `Position` and replace it with `None`.
    ///
    /// # Errors
    ///
    /// * `OutOfRange` if position is not valid
    /// * `AlreadyOccupied` if Position
    pub fn create_content(&mut self,position:Position,mut new_content:T) -> Result<()> {
        new_content.set_position(position);
        let index = try!(self.pos_to_index(position));
        try!(
            match self.position_status(position) {
                Ok(PositionStatus::Empty) => Ok(()),
                Ok(PositionStatus::Busy) => Err(Error::new(Reason::AlreadyOccupied)),
                Ok(PositionStatus::Forbidden) => Err(Error::new(Reason::ForbiddenLocation)),
                Err(_) => unreachable!()
            }
        );
        let ref mut content = self.contents_slice[index];
        match *content {
            None => {
                new_content.set_position(position);
                *content = Some(new_content);
                Ok(())
            },
            Some(_) => {
                Err(Error::new(Reason::AlreadyOccupied))
            }
        }
    }

    /// Swap 2 elements.
    ///
    /// # Errors
    ///
    /// * `OutOfRange` if one or more positions is not valid
    /// * `MissingTarget` if one the 2 position has no content (`None`)
    pub fn swap_contents(&mut self,pos_1:Position,pos_2:Position) -> Result<()> {
        let index_1 = try!(self.pos_to_index(pos_1));
        let index_2 = try!(self.pos_to_index(pos_2));
        let missing_target : bool = {
            let ref content_1 = self.contents_slice[index_1];
            let ref content_2 = self.contents_slice[index_2];
            content_1.is_none() || content_2.is_none()
        };
        if missing_target {
            Err(Error::new(Reason::MissingTarget))
        } else {
            self.contents_slice.swap(index_1,index_2);
            self.contents_slice[index_1].as_mut().unwrap().set_position(pos_2);
            self.contents_slice[index_2].as_mut().unwrap().set_position(pos_1);
            Ok(())
        }
    }

    /// Move an element from a position to another
    ///
    /// # Errors
    ///
    /// * `OutOfRange` if one or more positions are not valid
    /// * `MissingTarget` if the initial position has no element
    /// * `AlreadyOccupied` if the final position is busy
    pub fn move_contents(&mut self,from:Position,to:Position) -> Result<()> {
        let index_from = try!(self.pos_to_index(from));
        let index_to = try!(self.pos_to_index(to));
        let status_to = try!(self.position_status(to));
        try!(
            match status_to {
                PositionStatus::Empty => Ok(()),
                PositionStatus::Busy => Err(Error::new(Reason::AlreadyOccupied)),
                PositionStatus::Forbidden => Err(Error::new(Reason::ForbiddenLocation ))
            }
        );
        if self.contents_slice[index_from].is_none() {
            Err(Error::new(Reason::MissingTarget))
        } else if self.contents_slice[index_to].is_some() {
            Err(Error::new(Reason::AlreadyOccupied))
        } else {
            self.contents_slice.swap(index_from,index_to);
            self.contents_slice[index_to].as_mut().unwrap().set_position(to);
            Ok(())
        }
    }

    pub fn iter_contents(&self) -> MapIter<Iter<Option<T>>> {
        MapIter::new(self.contents_slice.iter(),self.length, self.offset)
    }

    pub fn iter_contents_mut(&mut self) -> MapIter<IterMut<Option<T>>> {
        MapIter::new(self.contents_slice.iter_mut(),self.length, self.offset)
    }

    pub fn iter_bg(&self) -> MapIter<Iter<Bg>> {
        MapIter::new(self.bg_slice.iter(),self.length, self.offset)
    }

    pub fn iter_bg_mut(&mut self) -> MapIter<IterMut<Bg>> {
        MapIter::new(self.bg_slice.iter_mut(),self.length, self.offset)
    }

    pub fn iter(&self) -> MapIter<Zip<Iter<Option<T>>,Iter<Bg>>> {
        let zipped_iter = self.contents_slice.iter().zip(self.bg_slice.iter()) ;
        MapIter::new(zipped_iter,self.length, self.offset)
    }

    pub fn iter_mut(&mut self) -> MapIter<Zip<IterMut<Option<T>>,IterMut<Bg>>> {
        let zipped_iter = self.contents_slice.iter_mut().zip(self.bg_slice.iter_mut()) ;
        MapIter::new(zipped_iter,self.length, self.offset)
    }
}

fn index_to_pos(index:usize,length:(i32,i32),offset:Position) -> Result<Position> {
    debug_assert!(length.0 > 0 && length.1 > 0);
    if index >= (length.0 * length.1) as usize {
        Err(Error::new(Reason::OutOfRange))
    } else {
        let y = index as i32 / length.0 ;
        let x = index as i32 % length.0 ;
        Ok(Position::new(x,y) + Position::from(offset))
    }
}


#[test]
pub fn test_pos_to_index(){
    let m = self::tests::sample_map();
    assert_eq!(m.pos_to_index(Position::new(-5,-5)).unwrap(),
               0);
    assert_eq!(m.pos_to_index(Position::new(-5,-4)).unwrap(),
               10);
    assert_eq!(m.pos_to_index(Position::new(-4,-4)).unwrap(),
               11);
    assert_eq!(m.pos_to_index(Position::new(4,4)).unwrap(),
               99);
    assert_eq!(m.pos_to_index(Position::new(4,5)).unwrap_err(),
               Error::new(Reason::OutOfRange));
    assert_eq!(m.pos_to_index(Position::new(5,0)).unwrap_err(),
               Error::new(Reason::OutOfRange));
    assert_eq!(m.pos_to_index(Position::new(-10,0)).unwrap_err(),
               Error::new(Reason::OutOfRange));
}

#[test]
pub fn test_index_to_pos(){
    let m = self::tests::sample_map();
    assert_eq!(m.index_to_pos(0).unwrap(),
               Position::new(-5,-5));
    assert_eq!(m.index_to_pos(11).unwrap(),
               Position::new(-4,-4));
    assert_eq!(m.index_to_pos(99).unwrap(),
               Position::new(4,4));
    assert_eq!(m.index_to_pos(100).unwrap_err(),
               Error::new(Reason::OutOfRange));
    assert_eq!(m.index_to_pos(150).unwrap_err(),
               Error::new(Reason::OutOfRange));
}

#[cfg(test)]
mod tests {
    use super::*;
    use pos::Position;
    use error::*;
    use std::string::String;
    #[derive(Debug)]
    pub struct Dummy {
        pub pos:Position,
        pub name:String
    }

    #[derive(Debug,Default)]
    pub struct Bg {
        pub kind:String
    }

    impl AllowContent for Bg {
        fn is_content_allowed(&self) -> bool {
            if self.kind == "Obstacle" {
                false
            } else {
                true
            }
        }
    }

    impl PositionAccessor for Dummy {
        fn set_position(&mut self,new_position:Position) {
            self.pos = new_position;
        }
        fn get_position(&self) -> Position {
            self.pos
        }
    }

    pub fn sample_map() -> Map<Dummy,Bg> {
        Map::new((10,10),Position::new(-5,-5)).unwrap()
    }

    #[test]
    fn routine_test(){
        let mut map : Map<Dummy,Bg> = sample_map();
        let dummy_1 = Dummy{
            pos:Position::default(),
            name:String::from("test_dummy_1")
        };
        let dummy_2 = Dummy {
            pos:Position::default(),
            name:String::from("test_dummy_2")
        };
        let dummy_3 = Dummy {
            pos:Position::default(),
            name:String::from("test_dummy_3")
        };
        let dummy_4 = Dummy {
            pos:Position::default(),
            name:String::from("test_dummy_4")
        };
        map.create_content(Position::new(0,0),dummy_1).unwrap();
        map.create_content(Position::new(2,0),dummy_2).unwrap();
        assert_eq!(map.get_contents(Position::new(2,0)).unwrap().as_ref().unwrap().get_position(),
                   Position::new(2,0));

        let err = map.create_content(Position::new(2,0),dummy_3);
        assert_eq!(err.unwrap_err(),
                   Error::new(Reason::AlreadyOccupied));

        map.get_bg_mut(Position::new(4,0)).unwrap().kind = String::from("Obstacle");

        let err = map.create_content(Position::new(4,0),dummy_4);
        assert_eq!(err.unwrap_err(),
                   Error::new(Reason::ForbiddenLocation));

        // create 2 dummies and swap their position
        map.swap_contents(Position::new(2,0), Position::new(0,0)).unwrap();
        assert_eq!(map.swap_contents(Position::new(3,0), Position::new(0,0)).unwrap_err(),
                   Error::new(Reason::MissingTarget));
        assert_eq!(map.iter_contents()
                      .filter(|&(_,ref dummy_option)| dummy_option.is_some())
                      .count(),
                   2);
        // count 2 dummies
        {
            let iter = map.iter_mut().filter(|&(_,(ref dummy,_))| dummy.is_some());
            for (pos,(mut opt, mut bg)) in iter {
                *opt = None ;
                // delete 2 dummies
            }
        }
        assert_eq!(map.iter_contents()
                      .filter(|&(_,ref dummy_option)| dummy_option.is_some())
                      .count(),
                   0);
        // count 0 dummies
    }
}
