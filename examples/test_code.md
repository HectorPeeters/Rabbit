# Code Syntax Highlighting

Demo of code highlighting

# Text

```
test code
```

# Rust

```Rust
fn consume_until(&mut self, f: fn(String) -> bool) -> String {
    let mut result = String::default();

    loop {
        if self.eof() {
            break;
        }

        let c = self.peek(0);

        if f(c) {
            break;
        }

        result.push_str(self.consume());
    }

    result
}
```

# Java

```Java
public void captureScreen(String fileName) throws Exception {
   Dimension screenSize = Toolkit.getDefaultToolkit().getScreenSize();
   Rectangle screenRectangle = new Rectangle(screenSize);
   Robot robot = new Robot();
   BufferedImage image = robot.createScreenCapture(screenRectangle);
   ImageIO.write(image, "png", new File(fileName));
}

public void test() {
  for (int x = 0; x < 100; x++) {
    System.out.println("Hello world!");
  }
}
```

# HTML

```Html
&lt;div class="sidenav">
  <a href="#about">About</a>
  <a href="#services">Services</a>
  <a href="#clients">Clients</a>
  <a href="#contact">Contact</a>
  <button class="dropdown-btn">Dropdown
    <i class="fa fa-caret-down"></i>
  </button>
  <div class="dropdown-container">
    <a href="#">Link 1</a>
    <a href="#">Link 2</a>
    <a href="#">Link 3</a>
  </div>
  <a href="#contact">Search</a>
</div>
```
