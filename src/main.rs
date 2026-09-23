use clap::Parser;
use std::fmt::{Debug, Write as fmtwrite};
use std::fs::OpenOptions;
use std::{
    fmt::Display,
    fs::File,
    io::{Read, Write},
    ops::{Deref, DerefMut},
    path::PathBuf,
};

const INSTURCTIONS_MSG: &'static str = include_str!("../INSTRUCTIONS");

#[derive(Parser, Debug)]
struct CliArgs {
    #[arg(long, value_name = "PATH")]
    out: Option<PathBuf>,

    #[arg(long("in"), value_name = "PATH")]
    _in: Option<PathBuf>,

    #[arg(long("stdin"), conflicts_with = "_in", value_name = "CODE")]
    stdin: Option<String>,

    #[arg(long, default_value_t = MachineCodeGenerationStrategy::HumanReadable)]
    strategy: MachineCodeGenerationStrategy,

    #[arg(long)]
    instructions: bool,
}

#[derive(Debug)]
enum LexicalInput {
    StdIn(String),
    File(File),
}

#[derive(Debug)]
enum GenerationalOutput {
    StdOut,
    File(File),
}

#[derive(Debug)]
struct Cli {
    _in: LexicalInput,
    out: GenerationalOutput,
    strategy: MachineCodeGenerationStrategy,
}

impl TryFrom<CliArgs> for Cli {
    type Error = CompilerError;

    fn try_from(value: CliArgs) -> Result<Self, Self::Error> {
        fn create_file(path: PathBuf, is_read: bool) -> CompilerResult<File> {
            OpenOptions::new()
                .read(is_read)
                .write(!is_read)
                .create(!is_read)
                .open(path)
                .map_err(|e| CompilerError::OsFileReadError(e))
        }

        let _in: LexicalInput = match value._in {
            Some(path) => LexicalInput::File(create_file(path, true)?),
            None => LexicalInput::StdIn(value.stdin.map_or(Err(CompilerError::NoInput), |s| Ok(s))?),
        };

        let out: GenerationalOutput = match value.out {
            Some(path) => GenerationalOutput::File(create_file(path, false)?),
            None => GenerationalOutput::StdOut,
        };

        Ok(Self {
            _in,
            out,
            strategy: value.strategy,
        })
    }
}

impl Cli {
    fn write_output(&mut self, text: &str) -> CompilerResult<()> {
        match &mut self.out {
            GenerationalOutput::StdOut => {
                println!("{}", text);
                Ok(())
            }
            GenerationalOutput::File(file) => {
                file.write_all(text.as_bytes()).map_err(|e| CompilerError::OsFileWriteError(e))?;
                Ok(())
            }
        }
    }

    fn read_input(&mut self) -> CompilerResult<String> {
        match &mut self._in {
            LexicalInput::StdIn(input) => Ok(input.to_string()),
            LexicalInput::File(file) => {
                let mut str = String::new();
                file.read_to_string(&mut str).map_err(|e| CompilerError::OsFileReadError(e))?;
                Ok(str)
            }
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
struct Token<'a>(&'a str);

type Tokens<'a> = Vec<Token<'a>>;

#[allow(dead_code)]
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
    OsFileReadError(std::io::Error),
    OsFileWriteError(std::io::Error),
    NoInput,
}

type CompilerResult<T> = Result<T, CompilerError>;

#[derive(clap::ValueEnum, Debug, Clone)]
enum MachineCodeGenerationStrategy {
    #[value(name = "pure")]
    Pure,

    #[value(name = "newline")]
    NewLineSepertion,

    #[value(name = "humanreadable")]
    HumanReadable,
}

trait GenerationStrategyTrait {
    fn generate(out: &mut String, instructions: Instructions);
}

struct Pure;
struct NewLineSeperation;
struct HumanReadable;

impl Display for MachineCodeGenerationStrategy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MachineCodeGenerationStrategy::Pure => f.write_str("pure"),
            MachineCodeGenerationStrategy::NewLineSepertion => f.write_str("newline"),
            MachineCodeGenerationStrategy::HumanReadable => f.write_str("humanreadable"),
        }
    }
}

impl GenerationStrategyTrait for Pure {
    fn generate(out: &mut String, instructions: Instructions) {
        *out += "0000000000000000";
        for instruction in instructions.0.into_iter() {
            instruction.generate(out);
        }
    }
}

impl GenerationStrategyTrait for NewLineSeperation {
    fn generate(out: &mut String, instructions: Instructions) {
        *out += "0000000000000000\n";
        for instruction in instructions.0.into_iter() {
            instruction.generate(out);
            *out += "\n";
        }
    }
}

impl GenerationStrategyTrait for HumanReadable {
    fn generate(out: &mut String, instructions: Instructions) {
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
            "or" => Ok(InstructionType::Or),
            "and" => Ok(InstructionType::And),
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
            _ => false,
        }
    }

    fn to_string(self, long_format: bool) -> String {
        match self {
            InstructionExpr::NONE => String::from("0000"),
            InstructionExpr::Register(register) => {
                if long_format {
                    format!("{:08b}", register as u8)
                } else {
                    format!("{:04b}", register as u8)
                }
            }
            InstructionExpr::Immediate(number, is_negative) => {
                if long_format {
                    format!(
                        "{:08b}",
                        if is_negative {
                            apply_twos_complement(number)
                        } else {
                            number
                        }
                    )
                } else {
                    format!(
                        "{:04b}",
                        if is_negative {
                            apply_twos_complement(number)
                        } else {
                            number
                        }
                    )
                }
            }
        }
    }
}

impl Instruction {
    fn _generate(self) -> (String, String, String) {
        let can_use_immediate = self._type.can_use_immediate();
        (
            format!(
                "{:04b}",
                self._type as u8
                    + (if self.operand.is_register() && can_use_immediate {
                        0
                    } else {
                        1
                    })
            ),
            self.dest.to_string(false),
            self.operand.to_string(true),
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
        match self {
            CompilerError::UnexpectedInstruction(instruction) => {
                f.write_fmt(format_args!("Unexpected instruction `{}`.", instruction))
            }
            CompilerError::NonNumericValueNotAccepted(value) => {
                f.write_fmt(format_args!("Numeric value expected here `{}`.", value))
            }
            CompilerError::ImmediateByteOverflow(value) => f.write_fmt(format_args!(
                "Number goes beyond byte limit (0..255/-128..127) `{}`.",
                value
            )),
            CompilerError::OsFileWriteError(e) => f.write_fmt(format_args!("Os file write error: {}", e)),
            CompilerError::OsFileReadError(e) => f.write_fmt(format_args!("Os file read error: {}", e)),
            CompilerError::NoInput => f.write_str("No input is given."),
        }
    }
}

impl Debug for CompilerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{}", self))
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

fn generate(
    instructions: Instructions,
    generation_strategy: &MachineCodeGenerationStrategy,
) -> String {
    match generation_strategy {
        MachineCodeGenerationStrategy::Pure => _generate::<Pure>(instructions),
        MachineCodeGenerationStrategy::NewLineSepertion => {
            _generate::<NewLineSeperation>(instructions)
        }
        MachineCodeGenerationStrategy::HumanReadable => _generate::<HumanReadable>(instructions),
    }
}

fn _assemble(mut cli: Cli) -> CompilerResult<()> {
    let input = cli.read_input()?;
    let tokens = lex(&input);
    let instructions = parse(tokens)?;

    instructions.resolve()?;
    let output = generate(instructions, &cli.strategy);
    cli.write_output(&output)?;

    Ok(())
}

fn _main() -> CompilerResult<bool> {
    let cli = CliArgs::parse();
    if cli.instructions {
        println!("{}", INSTURCTIONS_MSG);
        return Ok(false);
    }

    let cli: Cli = cli.try_into()?;
    _assemble(cli)?;
    Ok(true)
}

fn main() {
    match _main() {
        Ok(print) => {
            if print {
                println!("Compiled!");
            }
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
