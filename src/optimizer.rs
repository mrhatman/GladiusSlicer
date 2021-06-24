use crate::types::{Command, StateChange};
use itertools::{Itertools};
use std::rc::Rc;
use std::cell::RefCell;
use std::borrow::BorrowMut;
use std::ops::Deref;

pub fn optimize_commands(cmds: &mut Vec<Command> ) {

    let mut optimization_found = true;
    let mut size  = cmds.len();

    while {

        optimization_found = false;

        state_optomizer(cmds);
        unary_optimizer(cmds);
        binary_optimizer(cmds);

         cmds.len() != size

    }{
        size = cmds.len()
    }

}


pub fn unary_optimizer(cmds: &mut Vec<Command> ){



    cmds.retain(|cmd|{

        match cmd{
            Command::MoveTo { .. } => { true }
            Command::MoveAndExtrude { start, end } => { start != end }
            Command::LayerChange { .. } => {true }
            Command::SetState { new_state } => {
                !(new_state.ExtruderTemp.is_none() && new_state.MovementSpeed.is_none() && new_state.Retract.is_none() && new_state.ExtruderTemp.is_none() && new_state.BedTemp.is_none() )
            }
            Command::Delay { msec } => { *msec !=0}
            Command::Arc { start, end,.. } => {start != end}
        }

    });

}

pub fn binary_optimizer(cmds: &mut Vec<Command> ){



    *cmds = cmds.drain(..).coalesce(move |first,second|{

        match (first.clone(),second.clone()){
            (Command::MoveAndExtrude {start: f_start,end: f_end}, Command::MoveAndExtrude {start : s_start,end:s_end}) => {
                if f_end == s_start {
                    let det = (((f_start.x - s_start.x)*(s_start.y-s_end.y)) - ((f_start.y - s_start.y)*(s_start.x-s_end.x)) ).abs();

                    if det < 0.00001{
                        //Colinear

                        return Ok(Command::MoveAndExtrude { start: f_start, end: s_end });
                    }
                }
            }
            (Command::MoveTo {end: f_end}, Command::MoveTo {end:s_end}) => {

                        return Ok(Command::MoveTo { end: s_end });
            }

            (Command::SetState {new_state:f_state}, Command::SetState {new_state:s_state}) => {

                return Ok(Command::SetState {new_state: f_state.combine(&s_state)} );
            }
            (_, _) => {}
        }

        Err((first,second))
    }).collect();

}



pub fn state_optomizer(cmds: &mut Vec<Command> ){

    let mut current_state =StateChange::default();

    for cmd_ptr in  cmds{
        if let Command::SetState {new_state} =cmd_ptr{
            *new_state = current_state.state_diff(&new_state);
        }
    };


}


