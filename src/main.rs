use std::{env, fmt::Display, ops::{Deref, DerefMut}};
use std::fmt::Write;

#[derive(Debug, PartialEq, Eq)]
struct Token<'a>(&'a str);

type Tokens<'a> = Vec<Token<'a>>;

#[derive(Debug, Default)]
enum InstructionType {
    #[default]
    NOP = 0b0000,
    Halt = 0b0001,
    Move = 0b0010,
    Add = 0b0100,
    Sub = 0b0110,
    And = 0b1000,
    Or = 0b1010,
    Load = 0b1100,
    Store = 0b1110,
    Jump = 0b1111,
}

#[derive(Debug, Default)]
enum Register {
    #[default]
    RegA = 0b0000,
    RegB = 0b0001,
    RegC = 0b0010,
    RegD = 0b0011,
}

#[derive(Debug, Default)]
enum InstructionExpr {
    #[default]
    NONE,
    Register(Register),
    Immediate(u8, bool), //Value isNegative
}

#[derive(Debug, Default)]
struct Instruction {
    _type: InstructionType,
    dest: InstructionExpr,
    operand: InstructionExpr,
}

struct Instructions(Vec<Instruction>);

enum CompilerError {
    UnexpectedInstruction(String),
    NonNumericValueNotAccepted(String),
    ImmediateByteOverflow(String),
}

type CompilerResult<T> = Result<T, CompilerError>;

enum MachineCodeGenerationStrategy {
    Pure,
    NewLineSepertion,
    HumanReadable,
}

trait GenerationStrategyTrait {
    fn generate(out:&mut String, instructions: Instructions);
}

struct Pure;
struct NewLineSeperation;
struct HumanReadable;

impl GenerationStrategyTrait for Pure {
    fn generate(out:&mut String, instructions: Instructions) {
        todo!()
    }
}

impl GenerationStrategyTrait for NewLineSeperation {
    fn generate(out:&mut String, instructions: Instructions) {
        todo!()
    }
}

impl GenerationStrategyTrait for HumanReadable {
    fn generate(out:&mut String, instructions: Instructions) {
        *out += "0000 0000 00000000\n";
        for instruction in instructions.0.into_iter() {
            instruction.generate_with_space(out);
            *out += "\n";
        }
    }
}

impl<'a> Token<'a> {
    fn new(slice: &'a str) -> Self {
        Self(slice)
    }
}

impl<'a> PartialEq<&str> for Token<'a> {
    fn eq(&self, other: &&str) -> bool {
        self.0 == *other
    }
}

impl<'a> PartialEq<&str> for &Token<'a> {
    fn eq(&self, other: &&str) -> bool {
        *self.0 == **other
    }
}

impl TryFrom<&str> for Register {
    type Error = ();

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "RegA" => Ok(Register::RegA),
            "RegB" => Ok(Register::RegB),
            "RegC" => Ok(Register::RegC),
            "RegD" => Ok(Register::RegD),
            _ => Err(()),
        }
    }
}

impl TryFrom<&str> for InstructionType {
    type Error = CompilerError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "nop" => Ok(InstructionType::NOP),
            "halt" => Ok(InstructionType::Halt),
            "mov" => Ok(InstructionType::Move),
            "add" => Ok(InstructionType::Add),
            "sub" => Ok(InstructionType::Sub),
            "store" => Ok(InstructionType::Store),
            "load" => Ok(InstructionType::Load),
            "jmp" => Ok(InstructionType::Jump),
            _ => Err(CompilerError::UnexpectedInstruction(value.to_string())),
        }
    }
}

impl InstructionType {
    fn can_use_immediate(&self) -> bool {
        match self {
            Self::Jump | Self::NOP | Self::Halt => false,
            _ => true,
        }
    }
}

impl TryFrom<&str> for InstructionExpr {
    type Error = CompilerError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match Register::try_from(value) {
            Ok(register) => Ok(Self::Register(register)),
            Err(_) => {
                let numeric_value = value
                    .parse::<isize>()
                    .map_err(|_| CompilerError::NonNumericValueNotAccepted(value.to_string()))?;
                Ok(Self::Immediate(
                    numeric_value.abs().try_into().map_err(|_| {
                        CompilerError::ImmediateByteOverflow(numeric_value.abs().to_string())
                    })?,
                    numeric_value.is_negative(),
                ))
            }
        }
    }
}

impl InstructionExpr {
    fn is_register(&self) -> bool {
        match self {
            InstructionExpr::Register(_) => true,
            _ => false
        }
    }

    fn to_string(self, long_format: bool) -> String {
        match self {
            InstructionExpr::NONE => String::from("0000"),
            InstructionExpr::Register(register) => if long_format {format!("{:08b}", register as u8)} else {format!("{:04b}", register as u8)},
            InstructionExpr::Immediate(number, is_negative) => if long_format {format!("{:08b}", if is_negative {apply_twos_complement(number)} else {number})} else {format!("{:04b}", if is_negative {apply_twos_complement(number)} else {number})},
        }
    }
}

impl Instruction {
    fn _generate(self) -> (String, String, String) {
        let can_use_immediate = self._type.can_use_immediate();
        (
            format!("{:04b}", self._type as u8 +(if self.operand.is_register() && can_use_immediate {0} else {1})),
            self.dest.to_string(false),
            self.operand.to_string(true)
        )
    }

    fn generate(self, out: &mut String) {
        let (opcode, dest, operand) = Self::_generate(self);
        write!(out, "{}{}{}", opcode, dest, operand).unwrap();
    }

    fn generate_with_space(self, out: &mut String) {
        let (opcode, dest, operand) = Self::_generate(self);
        write!(out, "{} {} {}", opcode, dest, operand).unwrap();
    }
}

impl Deref for Instructions {
    type Target = Vec<Instruction>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Instructions {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Instructions {
    fn new() -> Self {
        Self(Vec::new())
    }

    fn resolve(&self) -> CompilerResult<()> {
        Ok(())
    }
}

impl Display for CompilerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let message: String = match self {
            CompilerError::UnexpectedInstruction(instruction) => format!("Unexpected instruction `{}`.", instruction),
            CompilerError::NonNumericValueNotAccepted(value) => format!("Numeric value expected here `{}`.", value),
            CompilerError::ImmediateByteOverflow(value) => format!("Number goes beyond byte limit (0..255/-128..127) `{}`.", value),
        };
        f.write_str(&message)
    }
}

fn apply_twos_complement(n: u8) -> u8 {
    n
}

fn lex<'a>(text: &'a str) -> Tokens<'a> {
    let mut cursor: usize = 0;
    let mut sequence_start: usize = 0;
    let mut tokens: Tokens<'a> = Tokens::new();
    let mut is_word = false;
    let bytes = text.as_bytes();
    while cursor < text.len() {
        let char = bytes[cursor];
        let is_token_end_char = char == b';';

        if is_token_end_char {
            if is_word {
                tokens.push(Token::new(&text[sequence_start..cursor]));
            }
            tokens.push(Token::new(&text[cursor..cursor + 1]));
            is_word = false;
            cursor += 1;
            continue;
        }

        if char.is_ascii_whitespace() {
            if is_word {
                tokens.push(Token::new(&text[sequence_start..cursor]));
                is_word = false;
            }
            cursor += 1;
            continue;
        }

        if !is_word {
            sequence_start = cursor;
            is_word = true;
        }
        cursor += 1;
    }
    tokens
}

fn parse<'a>(tokens: Tokens<'a>) -> CompilerResult<Instructions> {
    let mut instruction_phase = 0;
    let mut instrution: Instruction = Instruction::default();
    let mut instructions: Instructions = Instructions::new();

    for i in 0..tokens.len() {
        let token = tokens.get(i).unwrap();
        if token == ";" {
            instructions.push(instrution);
            instrution = Instruction::default();
            instruction_phase = 0;
            continue;
        }

        match instruction_phase {
            0 => instrution._type = InstructionType::try_from(token.0)?,
            1 => instrution.dest = InstructionExpr::try_from(token.0)?,
            2 => instrution.operand = InstructionExpr::try_from(token.0)?,
            _ => {}
        }

        instruction_phase += 1;
    }

    Ok(instructions)
}

fn _generate<T: GenerationStrategyTrait>(instructions: Instructions) -> String {
    let mut result = String::new();
    T::generate(&mut result, instructions);
    result
}

fn generate(instructions: Instructions, generation_strategy: MachineCodeGenerationStrategy) -> String {
    match generation_strategy {
        MachineCodeGenerationStrategy::Pure => _generate::<Pure>(instructions),
        MachineCodeGenerationStrategy::NewLineSepertion => _generate::<NewLineSeperation>(instructions),
        MachineCodeGenerationStrategy::HumanReadable => _generate::<HumanReadable>(instructions)
    }
}

fn _main(text: &String) -> CompilerResult<()> {
    let tokens = lex(&text);
    let instructions = parse(tokens)?;

    instructions.resolve()?;
    println!("{}", generate(instructions, MachineCodeGenerationStrategy::HumanReadable));

    Ok(())
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let text = args.get(1).expect("Need ASM code");
    match _main(text) {
        Ok(_) => {
            println!("Compiled!");
        }
        Err(err) => {
            println!("{}", err);
        }
    }
}

/*TODO:
1- Arity based parsing
2- Streaming formatters, (use std::gmt::Display) elimantes micro String allocations.
3- Add output File/Std choice support
4- Add input File/Std choice support
5- Fix Subtract bug
 */
