/// Built-in calculator (inspired by DN's CALC.PAS)
#[derive(Debug)]
pub struct Calculator {
    pub display: String,
    pub expression: String,
    pub result: f64,
    pub memory: f64,
    pub has_error: bool,
    pub new_entry: bool,
    pub last_op: Option<char>,
    pub accumulator: f64,
    pub history: Vec<String>,
}

impl Calculator {
    pub fn new() -> Self {
        Calculator {
            display: "0".to_string(),
            expression: String::new(),
            result: 0.0,
            memory: 0.0,
            has_error: false,
            new_entry: true,
            last_op: None,
            accumulator: 0.0,
            history: Vec::new(),
        }
    }

    pub fn press_digit(&mut self, d: char) {
        if self.has_error {
            self.clear();
        }
        if self.new_entry {
            self.display = String::new();
            self.new_entry = false;
        }
        if d == '.' && self.display.contains('.') {
            return;
        }
        if self.display == "0" && d != '.' {
            self.display = d.to_string();
        } else {
            self.display.push(d);
        }
    }

    pub fn press_op(&mut self, op: char) {
        if self.has_error {
            return;
        }
        self.evaluate_pending();
        self.accumulator = self.current_value();
        self.last_op = Some(op);
        self.new_entry = true;
        self.expression = format!("{} {} ", self.format_number(self.accumulator), op);
    }

    pub fn press_equals(&mut self) {
        if self.has_error {
            return;
        }
        self.evaluate_pending();
        let result_str = self.format_number(self.result);
        self.history.push(format!("{} = {}", self.expression, result_str));
        self.expression.clear();
        self.last_op = None;
        self.new_entry = true;
    }

    pub fn press_percent(&mut self) {
        if self.has_error {
            return;
        }
        let value = self.current_value();
        let pct = self.accumulator * value / 100.0;
        self.display = self.format_number(pct);
        self.result = pct;
    }

    pub fn press_negate(&mut self) {
        if self.has_error {
            return;
        }
        let value = self.current_value();
        self.display = self.format_number(-value);
    }

    pub fn press_sqrt(&mut self) {
        let value = self.current_value();
        if value < 0.0 {
            self.has_error = true;
            self.display = "Error".to_string();
            return;
        }
        self.display = self.format_number(value.sqrt());
        self.new_entry = true;
    }

    pub fn press_inverse(&mut self) {
        let value = self.current_value();
        if value == 0.0 {
            self.has_error = true;
            self.display = "Error".to_string();
            return;
        }
        self.display = self.format_number(1.0 / value);
        self.new_entry = true;
    }

    pub fn clear(&mut self) {
        self.display = "0".to_string();
        self.expression.clear();
        self.result = 0.0;
        self.has_error = false;
        self.new_entry = true;
        self.last_op = None;
        self.accumulator = 0.0;
    }

    pub fn clear_entry(&mut self) {
        self.display = "0".to_string();
        self.new_entry = true;
    }

    pub fn backspace(&mut self) {
        if self.new_entry || self.has_error {
            return;
        }
        self.display.pop();
        if self.display.is_empty() || self.display == "-" {
            self.display = "0".to_string();
            self.new_entry = true;
        }
    }

    pub fn memory_store(&mut self) {
        self.memory = self.current_value();
    }

    pub fn memory_recall(&mut self) {
        self.display = self.format_number(self.memory);
        self.new_entry = true;
    }

    pub fn memory_add(&mut self) {
        self.memory += self.current_value();
    }

    pub fn memory_clear(&mut self) {
        self.memory = 0.0;
    }

    fn evaluate_pending(&mut self) {
        if let Some(op) = self.last_op {
            let b = self.current_value();
            self.result = match op {
                '+' => self.accumulator + b,
                '-' => self.accumulator - b,
                '*' => self.accumulator * b,
                '/' => {
                    if b == 0.0 {
                        self.has_error = true;
                        self.display = "Error: Div/0".to_string();
                        return;
                    }
                    self.accumulator / b
                }
                '^' => self.accumulator.powf(b),
                _ => b,
            };
            self.display = self.format_number(self.result);
            self.accumulator = self.result;
        } else {
            self.result = self.current_value();
        }
    }

    fn current_value(&self) -> f64 {
        self.display.parse().unwrap_or(0.0)
    }

    fn format_number(&self, n: f64) -> String {
        if n == n.floor() && n.abs() < 1e15 {
            format!("{}", n as i64)
        } else {
            let s = format!("{:.10}", n);
            s.trim_end_matches('0').trim_end_matches('.').to_string()
        }
    }
}
