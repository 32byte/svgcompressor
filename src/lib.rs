extern crate roxmltree;

use roxmltree::{Attribute, Node};
use std::collections::HashMap;
use std::mem;
use std::io::Write;

/*
WARNING/TODOs:
 - When merging rect's its not accouted for namespaces
 - Rect equality checking is ugly due to the hardcoded keys
 - Need some debugging, which operations take the most time
 - Merging works only on rect's as of now.
 - The library needs some tests
*/

pub trait RectExt {
    fn get_x(&self) -> f64;
    fn get_y(&self) -> f64;
    fn get_w(&self) -> f64;
    fn get_h(&self) -> f64;

    fn same(&self, other: &Node, keys: &Vec<String>) -> bool;
    fn valid_rect(&self) -> bool;
}

impl RectExt for Node<'_, '_> {
    fn get_x(&self) -> f64 {
        self.attribute("x").unwrap().parse().unwrap()
    }

    fn get_y(&self) -> f64 {
        self.attribute("y").unwrap().parse().unwrap()
    }

    fn get_w(&self) -> f64 {
        self.attribute("width").unwrap().parse().unwrap()
    }

    fn get_h(&self) -> f64 {
        self.attribute("height").unwrap().parse().unwrap()
    }

    fn same(&self, other: &Node, keys: &Vec<String>) -> bool {
        for k in keys {
            let s = self.attribute(&k as &str);
            let o = other.attribute(&k as &str);

            if s.is_none() || o.is_none() || (s.unwrap() != o.unwrap()) {
                return false;
            }
        }
        true
    }

    fn valid_rect(&self) -> bool {
        self.is_element()
            && self.tag_name().name() == "rect"
            && self.attribute("x").is_some()
            && self.attribute("y").is_some()
            && self.attribute("width").is_some()
            && self.attribute("height").is_some()
    }
}

fn integer_decode(val: f64) -> (u64, i16, i8) {
    let bits: u64 = unsafe { mem::transmute(val) };
    let sign: i8 = if bits >> 63 == 0 { 1 } else { -1 };
    let mut exponent: i16 = ((bits >> 52) & 0x7ff) as i16;
    let mantissa = if exponent == 0 {
        (bits & 0xfffffffffffff) << 1
    } else {
        (bits & 0xfffffffffffff) | 0x10000000000000
    };

    exponent -= 1023 + 52;
    (mantissa, exponent, sign)
}

#[derive(Hash, Eq, PartialEq)]
struct Distance((u64, i16, i8));

impl Distance {
    fn new(val: f64) -> Distance {
        Distance(integer_decode(val))
    }
}

/*
pub fn nts(node: &Node) -> String {
    // ignores empty lines and comments
    if !node.is_element() && !node.is_text() {
        return "".to_string();
    }

    if node.is_text() {
        // roxmltree parses spaces between elements as text, which can be ignored
        if node
            .text()
            .unwrap()
            .to_string()
            .replace(" ", "")
            .replace("\n", "")
            == ""
        {
            return "".to_string();
        }

        return node.text().unwrap().to_string();
    }

    /*
    if node doesn't have children
    <node key="value">node_value</node>
    */
    let node_name = node.tag_name().name();

    // parse children
    let mut children = "".to_string();
    if node.has_children() {
        for e in node.children() {
            children += &nts(&e);
        }
    }

    // parse attributes
    let mut attributes = " ".to_string();
    for attr in node.attributes() {
        attributes += &format!("{}=\"{}\" ", attr.name(), attr.value());
    }
    // cut off last attributes char since it's a space
    attributes = attributes[0..attributes.len() - 1].to_string();

    let mut namespace = "".to_string();
    if node_name == "svg" {
        namespace = format!(
            " xmlns=\"{}\"",
            node.tag_name().namespace().unwrap().to_string()
        );
    }

    /* Note: To make the xml more clean this can be implemented
        I am not sure though if this might cause problems,
        so it's not implemented right now.

    if children == "" {
        return format!(
            "<{node}{attributes}{namespace} />",
            node = node_name,
            attributes = attributes,
            namespace = namespace
        );
    }
    */

    format!(
        "<{node}{attributes}{namespace}>{children}</{node}>",
        node = node_name,
        attributes = attributes,
        namespace = namespace,
        children = children
    )
}
*/

pub fn merge_rects(a: &Node, b: &Node) -> String {
    // merge children
    let children = children_to_string(a) + &children_to_string(b);

    // merge attributes
    let mut attributes_str = " ".to_string();
    let attributes: Vec<Attribute> = a.attributes().to_vec();
    for attr in attributes {
        let value = match attr.name() {
            "width" => {
                if a.get_y() == b.get_y() {
                    (a.get_w() + b.get_w()).to_string()
                } else {
                    a.get_w().to_string()
                }
            }
            "height" => {
                if a.get_x() == b.get_x() {
                    (a.get_h() + b.get_h()).to_string()
                } else {
                    a.get_h().to_string()
                }
            }
            _ => attr.value().to_string(),
        };

        attributes_str += &format!("{}=\"{}\" ", attr.name(), value);
    }
    attributes_str = attributes_str[0..attributes_str.len() - 1].to_string();

    format!(
        "<rect{attributes}>{children}</rect>",
        attributes = attributes_str,
        children = children
    )
}

pub fn children_to_string(node: &Node) -> String {
    let mut counter = 0;
    let mut c_merged = 0;

    let mut children_str = "".to_string();
    if node.has_children() {
        let mut rect_children: Vec<Node> = Vec::new();
        let mut other_children: Vec<Node> = Vec::new();
        let mut elem_blacklist: Vec<Node> = Vec::new();
        let mut x_map: HashMap<Distance, Vec<Node>> = HashMap::new();
        let mut y_map: HashMap<Distance, Vec<Node>> = HashMap::new();

        // Populate the x & y maps and children
        for e in node.children() {
            if !e.valid_rect() {
                other_children.push(e);
                continue;
            }

            match x_map.get_mut(&Distance::new(e.get_x())) {
                Some(v) => v.push(e),
                None => {
                    x_map.insert(Distance::new(e.get_x()), vec![e]);
                }
            }

            match y_map.get_mut(&Distance::new(e.get_y())) {
                Some(v) => v.push(e),
                None => {
                    y_map.insert(Distance::new(e.get_y()), vec![e]);
                }
            }

            rect_children.push(e);
        }

        // Sort rect_children by x and y
        rect_children.sort_by(|a, b| {
            if a.get_y() < b.get_y() {
                return std::cmp::Ordering::Less;
            }
            if a.get_y() == b.get_y() {
                return a.get_x().partial_cmp(&b.get_x()).unwrap();
            }
            return std::cmp::Ordering::Greater;
        });

        let all_ele_len = rect_children.len() + other_children.len();

        for e in &rect_children {
            let mut merged = false;

            counter += 1;
            if node.tag_name().name() == "svg" {
                print!("\r{}/{} {}/{}", counter, all_ele_len, (counter as i64 - other_children.len() as i64).max(0), other_children.len());
                std::io::stdout().flush().unwrap();
            }

            // This element was already compressed
            if elem_blacklist.contains(&e) {
                continue;
            }

            // check if there is element to the right
            // Note: Children have to be sorted since checking for neighbors to the left is impossible
            match x_map.get_mut(&Distance::new(e.get_x() + e.get_w())) {
                Some(neighbors) => {
                    for n in neighbors {
                        // neighbor was already compressed
                        if elem_blacklist.contains(n) {
                            continue;
                        }
                        if e.get_y() == n.get_y() && e.same(&n, &vec!["fill".to_string()]) {
                            children_str += &merge_rects(&e, n);

                            merged = true;

                            // "remove" e & c
                            elem_blacklist.push(*e);
                            elem_blacklist.push(*n);

                            break;
                        }
                    }
                }
                None => (),
            }

            // merged on the x-axis
            if merged {
                c_merged += 1;
                continue;
            }

            // check if there is element below
            // Note: Children have to be sorted since checking for neighbors above is impossible
            match y_map.get_mut(&Distance::new(e.get_y() + e.get_h())) {
                Some(neighbors) => {
                    for n in neighbors {
                        // neighbor was already compressed
                        if elem_blacklist.contains(n) {
                            continue;
                        }

                        if e.get_x() == n.get_x() && e.same(&n, &vec!["fill".to_string()]) {
                            children_str += &merge_rects(&e, n);

                            merged = true;

                            // "remove" e & c
                            elem_blacklist.push(*e);
                            elem_blacklist.push(*n);

                            break;
                        }
                    }
                }
                None => (),
            }

            // merged on the y-axis
            if merged {
                c_merged += 1;
                continue;
            }

            children_str += &compress_to_string(&e);
        }

        for e in &other_children {
            counter += 1;
            if node.tag_name().name() == "svg" {
                print!("\r{}/{} {}/{}", counter, all_ele_len, (counter as i64 - other_children.len() as i64).max(0), other_children.len());
                std::io::stdout().flush().unwrap();
            }

            children_str += &compress_to_string(e);
        }
    }

    if node.tag_name().name() == "svg" {
        println!("\nmerged {} elements", c_merged);
    }

    children_str
}

pub fn compress_to_string(node: &Node) -> String {
    // ignores empty lines and comments
    if !node.is_element() && !node.is_text() {
        return "".to_string();
    }

    if node.is_text() {
        // roxmltree parses spaces between elements as text, which can be ignored
        if node
            .text()
            .unwrap()
            .to_string()
            .replace(" ", "")
            .replace("\n", "")
            == ""
        {
            return "".to_string();
        }

        return node.text().unwrap().to_string();
    }

    let node_name = node.tag_name().name();

    // parse children
    let children = children_to_string(node);

    // parse attributes
    let mut attributes = " ".to_string();
    for attr in node.attributes() {
        attributes += &format!("{}=\"{}\" ", attr.name(), attr.value());
    }
    // cut off last attributes char since it's a space
    attributes = attributes[0..attributes.len() - 1].to_string();

    // parse namespace
    let mut namespace = "".to_string();
    if node_name == "svg" {
        namespace = format!(
            " xmlns=\"{}\"",
            node.tag_name().namespace().unwrap().to_string()
        );
    }

    /* Note: To make the xml more clean this can be implemented
        I am not sure though if this might cause problems,
        so it's not implemented right now.

    if children == "" {
        return format!(
            "<{node}{attributes}{namespace} />",
            node = node_name,
            attributes = attributes,
            namespace = namespace
        );
    }
    */

    format!(
        "<{node}{attributes}{namespace}>{children}</{node}>",
        node = node_name,
        attributes = attributes,
        namespace = namespace,
        children = children
    )
}

pub fn compress(content: &str) -> String {
    let doc = roxmltree::Document::parse(&content).expect("Couldn't parse svg");

    compress_to_string(&doc.root().first_child().unwrap())
}
/*
pub fn node_to_string(node: &roxmltree::Node) -> String {
    let mut attr_string = String::new();
    for attr in node.attributes() {
        attr_string += &format!("{}='{}' ", attr.name(), attr.value());
    }
    // TODO: clean up this scuffed mess
    if node.tag_name().name() == "text" {
        format!(
            "<{} {}>{}</{}>",
            node.tag_name().name(),
            &attr_string[0..attr_string.len() - 1],
            node.text().unwrap(),
            node.tag_name().name()
        )
    } else if node.tag_name().name() == "svg" {
        format!(
            "<{} {} xmlns='http://www.w3.org/2000/svg' version='1.1' xmlns:xlink='http://www.w3.org/1999/xlink'>",
            node.tag_name().name(),
            format!("width='{}' height='{}'", node.attribute("width").unwrap(), node.attribute("height").unwrap())
        )
    } else {
        format!("<{} {}/>", node.tag_name().name(), attr_string)
    }
}

pub fn compress(input_path: &str, output_path: &str) {
    let content = fs::read_to_string(input_path).expect("File not found!");

    let doc = roxmltree::Document::parse(&content).expect("Couldn't parse svg");
    let mut output = String::new();

    // Keys for the equality check
    let keys: Vec<String> = vec!["fill".to_string()];

    let mut children: Vec<Node> = Vec::new();
    let mut elem_blacklist: Vec<Node> = Vec::new();
    let mut x_map: HashMap<Distance, Vec<Node>> = HashMap::new();
    let mut y_map: HashMap<Distance, Vec<Node>> = HashMap::new();
    let mut non_rect_nodes: Vec<Node> = Vec::new();

    // Populate the x & y maps and children
    for e in doc.root().first_child().unwrap().children() {
        if !e.is_element() {
            continue;
        }

        if e.tag_name().name() != "rect" {
            non_rect_nodes.push(e);
            continue;
        }

        match x_map.get_mut(&Distance::new(e.get_x())) {
            Some(v) => v.push(e),
            None => {
                x_map.insert(Distance::new(e.get_x()), vec![e]);
            }
        }

        match y_map.get_mut(&Distance::new(e.get_y())) {
            Some(v) => v.push(e),
            None => {
                y_map.insert(Distance::new(e.get_y()), vec![e]);
            }
        }

        children.push(e);
    }

    // Sort children by x and y
    children.sort_by(|a, b| {
        if a.get_y() < b.get_y() {
            return std::cmp::Ordering::Less;
        }
        if a.get_y() == b.get_y() {
            return a.get_x().partial_cmp(&b.get_x()).unwrap();
        }
        return std::cmp::Ordering::Greater;
    });

    for c in &mut children {
        if elem_blacklist.contains(c) {
            continue;
        }

        // check if there is element to the right
        // Note: Children have to be sorted since checking for neighbors to the left is impossible
        match x_map.get_mut(&Distance::new(c.get_x() + c.get_w())) {
            Some(neighbors) => {
                for e in neighbors {
                    if elem_blacklist.contains(e) {
                        continue;
                    }

                    if c.get_y() == e.get_y() && c.same(&e, &keys) {
                        // TODO: merge

                        // "remove" e & c
                        elem_blacklist.push(*e);
                        elem_blacklist.push(*c);

                        break;
                    }
                }
            }
            None => (),
        }
    }

    //let mut rect: Option<roxmltree::Node> = None;

    output += &node_to_string(&doc.root().first_child().unwrap());
    output += "\n";

    // Pseudo-code
    /*
    let x_map = hashmap(children, key='x');
    let y_map = hashmap(children, key='y');

    for c in children {
        // check to the right
        for e in x_map[c.x+c.w] {
            if e.y == c.y {
                merge(e, c)
                x_map.remove(e)
                break;
            }
        }

        // check to the bottom
        for e in y_map[c.y+c.h] {
            if e.x == c.x {
                merge(e, c)
                y_map.remove(e)
                break;
            }
        }
    }
    */

    // Assumes there are no nested tags
    /*
    for elem in doc.root().first_child().unwrap().children() {
        if !elem.is_element() {
            continue;
        }

        // Lets for first only compress rectangles
        if elem.tag_name().name() == "rect" {
            if rect.is_none() {
                rect = Some(elem);
                continue;
            } else {
                // Rect is some, try to connect rect and elem
                let rx: f32 = rect.unwrap().attribute("x").unwrap().parse().unwrap();
                let ry: f32 = rect.unwrap().attribute("y").unwrap().parse().unwrap();
                let rw: f32 = rect.unwrap().attribute("width").unwrap().parse().unwrap();
                let rh: f32 = rect.unwrap().attribute("height").unwrap().parse().unwrap();

                let ex: f32 = elem.attribute("x").unwrap().parse().unwrap();
                let ey: f32 = elem.attribute("y").unwrap().parse().unwrap();
                let ew: f32 = elem.attribute("width").unwrap().parse().unwrap();
                let eh: f32 = elem.attribute("height").unwrap().parse().unwrap();

                // Above or below each other
                if rx == ex {
                    println!("Above or below");
                    // rect above elem
                    if ry + rh == ey + 1. {
                        println!("rect above elem");
                        rect = None;
                    }
                } else {
                    println!("{} {}", rx, ex)
                }
            }
        }
        output += &node_to_string(&elem);
        output += "\n";
    }
    */
    output += "</svg>";

    let elem = doc
        .descendants()
        .find(|n| n.tag_name().name() == "rect")
        .unwrap();

    println!("{:?}", &elem);
    println!("{}", node_to_string(&elem));

    fs::write(output_path, output).expect("Couldn't write to file!");
}
 */