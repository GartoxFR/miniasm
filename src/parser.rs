use nom::branch::alt;
use nom::bytes::complete::tag;
use nom::character::complete::{self, alphanumeric1, anychar, line_ending, space0, space1};
use nom::combinator::{eof, map, peek};
use nom::multi::{many0, many_till, separated_list0};
use nom::sequence::{delimited, preceded, terminated};
use nom::IResult;

use crate::types::{DataHolder, Instruction, JRCond, JumpTarget, Line, Operation};

pub fn sign_or_unsigned_int8(input: &str) -> IResult<&str, i8> {
    alt((complete::i8, map(complete::u8, |val| val as i8)))(input)
}

pub fn reg_a(input: &str) -> IResult<&str, DataHolder> {
    tag("A")(input).map(|(input, _)| (input, DataHolder::A))
}

pub fn reg_b(input: &str) -> IResult<&str, DataHolder> {
    tag("B")(input).map(|(input, _)| (input, DataHolder::B))
}

pub fn cst(input: &str) -> IResult<&str, DataHolder> {
    map(sign_or_unsigned_int8, DataHolder::Const)(input)
}

pub fn arg1(input: &str) -> IResult<&str, DataHolder> {
    alt((reg_a, reg_b))(input)
}

pub fn arg2(input: &str) -> IResult<&str, DataHolder> {
    alt((reg_a, cst))(input)
}

pub fn arg2_addr(input: &str) -> IResult<&str, DataHolder> {
    let (input, _) = tag("*")(input)?;

    arg2(input).map(|(input, val)| {
        (
            input,
            match val {
                DataHolder::A => DataHolder::AAddr,
                DataHolder::Const(x) => DataHolder::ConstAddr(x),
                _ => unreachable!(),
            },
        )
    })
}

pub fn add(input: &str) -> IResult<&str, Operation> {
    let (input, arg1) = arg1(input)?;
    let (input, _) = delimited(space0, tag("+"), space0)(input)?;
    let (input, arg2) = arg2(input)?;

    Ok((input, Operation::Add(arg1, arg2)))
}

pub fn sub(input: &str) -> IResult<&str, Operation> {
    let (input, arg1) = arg1(input)?;
    let (input, _) = delimited(space0, tag("-"), space0)(input)?;
    let (input, arg2) = arg2(input)?;

    Ok((input, Operation::Sub(arg1, arg2)))
}

pub fn and(input: &str) -> IResult<&str, Operation> {
    let (input, arg1) = arg1(input)?;
    let (input, _) = delimited(space0, tag("and"), space0)(input)?;
    let (input, arg2) = arg2(input)?;

    Ok((input, Operation::And(arg1, arg2)))
}

pub fn or(input: &str) -> IResult<&str, Operation> {
    let (input, arg1) = arg1(input)?;
    let (input, _) = delimited(space0, tag("or"), space0)(input)?;
    let (input, arg2) = arg2(input)?;

    Ok((input, Operation::Or(arg1, arg2)))
}

pub fn xor(input: &str) -> IResult<&str, Operation> {
    let (input, arg1) = arg1(input)?;
    let (input, _) = delimited(space0, tag("xor"), space0)(input)?;
    let (input, arg2) = arg2(input)?;

    Ok((input, Operation::Xor(arg1, arg2)))
}

pub fn lsr(input: &str) -> IResult<&str, Operation> {
    let (input, _) = tag("LSR")(input)?;
    let (input, _) = space1(input)?;
    let (input, arg1) = arg1(input)?;

    Ok((input, Operation::LShiftRight(arg1)))
}

pub fn not(input: &str) -> IResult<&str, Operation> {
    preceded(preceded(tag("not"), space1), arg1)(input)
        .map(|(input, val)| (input, Operation::Not(val)))
}

pub fn none(input: &str) -> IResult<&str, Operation> {
    alt((arg1, arg2, arg2_addr))(input).map(|(input, val)| (input, Operation::None(val)))
}

pub fn operation(input: &str) -> IResult<&str, Operation> {
    alt((add, sub, and, or, xor, lsr, not, none))(input)
}

pub fn assignment(input: &str) -> IResult<&str, Instruction> {
    let (input, op) = operation(input)?;
    let (input, _) = delimited(space0, tag("->"), space0)(input)?;
    let (input, dest) = alt((arg1, arg2_addr))(input)?;

    Ok((input, Instruction::Assignment { op, dest }))
}

pub fn jr_cond(input: &str) -> IResult<&str, JRCond> {
    alt((
        map(preceded(space1, tag("IFZ")), |_| JRCond::IfZ),
        map(preceded(space1, tag("IFC")), |_| JRCond::IfC),
        map(preceded(space1, tag("IFN")), |_| JRCond::IfN),
        map(preceded(space0, peek(alt((line_ending, tag("#"))))), |_| {
            JRCond::True
        }),
    ))(input)
}

pub fn const_jump_target(input: &str) -> IResult<&str, JumpTarget> {
    map(sign_or_unsigned_int8, |val| JumpTarget::Const(val as u8))(input)
}

pub fn label_jump_target(input: &str) -> IResult<&str, JumpTarget> {
    map(alphanumeric1, JumpTarget::Label)(input)
}

pub fn jr(input: &str) -> IResult<&str, Instruction> {
    let (input, _) = tag("JR")(input)?;
    let (input, _) = space1(input)?;
    let (input, val) = alt((const_jump_target, label_jump_target))(input)?;
    let (input, cond) = jr_cond(input)?;

    Ok((input, Instruction::JR(cond, val)))
}

pub fn ja(input: &str) -> IResult<&str, Instruction> {
    let (input, _) = tag("JA")(input)?;
    let (input, _) = space1(input)?;

    map(alt((const_jump_target, label_jump_target)), |val| {
        Instruction::JA(val)
    })(input)
}

pub fn check(input: &str) -> IResult<&str, Instruction> {
    let (input, arg1) = arg1(input)?;
    let (input, _) = delimited(space0, tag("-"), space0)(input)?;
    let (input, arg2) = arg2(input)?;
    let (input, _) = preceded(space0, tag("?"))(input)?;

    Ok((input, Instruction::Check(arg1, arg2)))
}

pub fn ret_i(input: &str) -> IResult<&str, Instruction> {
    map(tag("reti"), |_| Instruction::RetI)(input)
}

pub fn instruction(input: &str) -> IResult<&str, Instruction> {
    alt((jr, assignment, ja, check, ret_i))(input)
}

pub fn l(input: &str) -> IResult<&str, &str> {
    terminated(alphanumeric1, tag(":"))(input)
}

pub fn comment(input: &str) -> IResult<&str, ()> {
    let result: IResult<_, _> = preceded(space0, tag("#"))(input);

    match result {
        Ok((input, _)) => map(many_till(anychar, peek(line_ending)), |_| ())(input),
        Err(_) => Ok((input, ())),
    }
}

pub fn line(input: &str) -> IResult<&str, Line> {
    let (input, label) = map(l, Some)(input).unwrap_or((input, None));

    let (input, _) = many0(alt((space1, line_ending)))(input)?;
    let (input, instr) = instruction(input)?;
    let (input, _) = comment(input)?;

    Ok((input, (label, instr)))
}

pub fn parse_program(input: &str) -> IResult<&str, Vec<Line>> {
    let (input, program) =
        separated_list0(many0(line_ending), delimited(space0, line, space0))(input)?;
    let (input, _) = preceded(many0(alt((space1, line_ending))), eof)(input)?;
    Ok((input, program))
}
