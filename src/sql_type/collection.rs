use crate::sql_type::{Collection, FromSql};
use crate::ErrorKind;
use crate::Result;
use std::iter::FusedIterator;
use std::marker::PhantomData;

#[derive(Clone, Debug)]
enum State {
    Begin,
    Current(i32),
    End,
}

impl State {
    fn next(&self, coll: &Collection) -> Result<State> {
        let index_result = match self {
            State::Begin => coll.first_index(),
            State::Current(index) => coll.next_index(*index),
            State::End => return Ok(State::End),
        };
        match index_result {
            Ok(index) => Ok(State::Current(index)),
            Err(err) if err.kind() == ErrorKind::NoDataFound => Ok(State::End),
            Err(err) => Err(err),
        }
    }

    fn next_back(&self, coll: &Collection) -> Result<State> {
        let index_result = match self {
            State::Begin => return Ok(State::Begin),
            State::Current(index) => coll.prev_index(*index),
            State::End => coll.last_index(),
        };
        match index_result {
            Ok(index) => Ok(State::Current(index)),
            Err(err) if err.kind() == ErrorKind::NoDataFound => Ok(State::Begin),
            Err(err) => Err(err),
        }
    }
}

/// An iterator over the elements of a Collection.
///
/// This struct is created by [`Collection::iter()`]. See its documentation for more.
#[derive(Clone, Debug)]
pub struct Iter<'a, T: FromSql> {
    coll: &'a Collection,
    state: State,
    phantom: PhantomData<T>,
}

impl<T: FromSql> Iter<'_, T> {
    fn try_next(&mut self) -> Result<Option<(i32, T)>> {
        let next_state = self.state.next(self.coll)?;
        let result = if let State::Current(index) = next_state {
            Some((index, self.coll.get(index)?))
        } else {
            None
        };
        self.state = next_state;
        Ok(result)
    }

    fn try_next_back(&mut self) -> Result<Option<(i32, T)>> {
        let next_state = self.state.next_back(self.coll)?;
        let result = if let State::Current(index) = next_state {
            Some((index, self.coll.get(index)?))
        } else {
            None
        };
        self.state = next_state;
        Ok(result)
    }
}

impl<T: FromSql> Iter<'_, T> {
    pub(crate) fn new(coll: &Collection) -> Iter<T> {
        Iter {
            coll,
            state: State::Begin,
            phantom: PhantomData,
        }
    }
}

impl<T> Iterator for Iter<'_, T>
where
    T: FromSql,
{
    type Item = Result<(i32, T)>;
    fn next(&mut self) -> Option<Result<(i32, T)>> {
        self.try_next().transpose()
    }
}

impl<T> DoubleEndedIterator for Iter<'_, T>
where
    T: FromSql,
{
    fn next_back(&mut self) -> Option<Result<(i32, T)>> {
        self.try_next_back().transpose()
    }
}

impl<T: FromSql> FusedIterator for Iter<'_, T> {}

/// An iterator over the values of a Collection.
///
/// This struct is created by [`Collection::values()`]. See its documentation for more.
#[derive(Clone, Debug)]
pub struct Values<'a, T: FromSql> {
    coll: &'a Collection,
    state: State,
    phantom: PhantomData<T>,
}

impl<T: FromSql> Values<'_, T> {
    fn try_next(&mut self) -> Result<Option<T>> {
        let next_state = self.state.next(self.coll)?;
        let result = if let State::Current(index) = next_state {
            Some(self.coll.get(index)?)
        } else {
            None
        };
        self.state = next_state;
        Ok(result)
    }

    fn try_next_back(&mut self) -> Result<Option<T>> {
        let next_state = self.state.next_back(self.coll)?;
        let result = if let State::Current(index) = next_state {
            Some(self.coll.get(index)?)
        } else {
            None
        };
        self.state = next_state;
        Ok(result)
    }
}

impl<T: FromSql> Values<'_, T> {
    pub(crate) fn new(coll: &Collection) -> Values<T> {
        Values {
            coll,
            state: State::Begin,
            phantom: PhantomData,
        }
    }
}

impl<T> Iterator for Values<'_, T>
where
    T: FromSql,
{
    type Item = Result<T>;
    fn next(&mut self) -> Option<Result<T>> {
        self.try_next().transpose()
    }
}

impl<T> DoubleEndedIterator for Values<'_, T>
where
    T: FromSql,
{
    fn next_back(&mut self) -> Option<Result<T>> {
        self.try_next_back().transpose()
    }
}

impl<T: FromSql> FusedIterator for Values<'_, T> {}

/// An iterator over the indices of a Collection.
///
/// This struct is created by [`Collection::indices()`]. See its documentation for more.
#[derive(Clone, Debug)]
pub struct Indices<'a> {
    coll: &'a Collection,
    state: State,
}

impl Indices<'_> {
    pub(crate) fn new(coll: &Collection) -> Indices {
        Indices {
            coll,
            state: State::Begin,
        }
    }

    fn try_next(&mut self) -> Result<Option<i32>> {
        self.state = self.state.next(self.coll)?;
        Ok(if let State::Current(index) = self.state {
            Some(index)
        } else {
            None
        })
    }

    fn try_next_back(&mut self) -> Result<Option<i32>> {
        self.state = self.state.next_back(self.coll)?;
        Ok(if let State::Current(index) = self.state {
            Some(index)
        } else {
            None
        })
    }
}

impl Iterator for Indices<'_> {
    type Item = Result<i32>;
    fn next(&mut self) -> Option<Result<i32>> {
        self.try_next().transpose()
    }
}

impl DoubleEndedIterator for Indices<'_> {
    fn next_back(&mut self) -> Option<Result<i32>> {
        self.try_next_back().transpose()
    }
}

impl FusedIterator for Indices<'_> {}
