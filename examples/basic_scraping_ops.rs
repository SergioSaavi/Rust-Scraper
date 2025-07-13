use std::time::Duration;
use futures::StreamExt;
use chromiumoxide::{Browser, BrowserConfig};
use chromiumoxide::handler::Handler;
use chromiumoxide::cdp::browser_protocol::page::CaptureScreenshotParams;

// Main function using async-std runtime
#[async_std::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Starting web scraper...");

    // Create a browser with custom configuration
    let (mut browser, mut handler) = create_browser().await?;

    // Run the browser handler in a separate task
    let handle = async_std::task::spawn(async move {
        while let Some(_event) = handler.next().await {
            // You could process browser events here if needed
            // println!("Browser event: {:?}", event);
        }
    });

    // Example 1: Basic page navigation and content extraction
    println!("\n--- Example 1: Basic Wikipedia Search ---");
    let result = wikipedia_search(&mut browser, "Rust programming language").await?;
    println!("Search result title: {}", result);

    // Example 2: Taking screenshots
    println!("\n--- Example 2: Taking Screenshots ---");
    take_screenshot(&mut browser, "https://rust-lang.org", "rust-homepage.png").await?;

    // Example 3: Extracting structured data
    println!("\n--- Example 3: Extracting Structured Data ---");
    let books = extract_books(&mut browser).await?;
    println!("Found {} books:", books.len());
    for (i, book) in books.iter().enumerate().take(5) {
        println!("{}. {}", i+1, book);
    }
    if books.len() > 5 {
        println!("... and {} more", books.len() - 5);
    }

    // Clean up
    browser.close().await?;
    handle.await;
    println!("\nScraper finished successfully!");
    Ok(())
}

// Helper function to create a browser with proper configuration
async fn create_browser() -> Result<(Browser, Handler), Box<dyn std::error::Error>> {
    // Configure the browser
    let config = BrowserConfig::builder() // Show the browser UI
        .window_size(1280, 800) // Set default window size (fixes mobile viewport)
        .args(vec!["--disable-blink-features=AutomationControlled"]) // Avoid detection
        .build()?;

    // Launch the browser
    Ok(Browser::launch(config).await?)
}

// Example function for Wikipedia search
async fn wikipedia_search(browser: &mut Browser, search_term: &str) -> Result<String, Box<dyn std::error::Error>> {
    // Create a new page with specific viewport size
    let page = browser.new_page("https://en.wikipedia.org").await?;

    // Wait for page to load
    page.wait_for_navigation().await?;

    // Find and click on the search button
    let search_button = page.find_element("#p-search > a").await?;
    search_button.click().await?;

    // Wait a moment to ensure the input field is ready
    async_std::task::sleep(Duration::from_millis(500)).await;

    // Find the search input field
    let input_selector = "input[name='search']";
    let search_input = page.find_element(input_selector).await?;

    // Fix for type_str deleting first character - first click to focus, then insert text
    search_input.click().await?;
    // Wait a bit after clicking
    async_std::task::sleep(Duration::from_millis(100)).await;

    // Type the search term
    search_input.type_str(search_term).await?;

    // Use keyboard to press Enter - through JavaScript since we don't have press_key
    page.evaluate(r#"
        document.querySelector('input[name="search"]').form.submit();
    "#).await?;

    // Wait for the search results page to load
    page.wait_for_navigation().await?;

    // Extract the title of the page
    let title = page.evaluate("document.title").await?;
    let title_str: String = title.into_value()?;

    Ok(title_str)
}

// Example function for taking screenshots
async fn take_screenshot(browser: &mut Browser, url: &str, filename: &str) -> Result<(), Box<dyn std::error::Error>> {
    let page = browser.new_page(url).await?;

    // Wait for the page to fully load
    page.wait_for_navigation().await?;

    // Additional wait for any JavaScript to execute
    async_std::task::sleep(Duration::from_secs(1)).await;

    // Take a screenshot and save it to a file
    // Using the proper params struct instead of a boolean
    let screenshot_params = CaptureScreenshotParams::default();
    let screenshot_data = page.screenshot(screenshot_params).await?;
    std::fs::write(filename, screenshot_data)?;

    println!("Screenshot saved to {}", filename);
    Ok(())
}

// Example function to extract structured data from a page
async fn extract_books(browser: &mut Browser) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    // Visit a book listing page - books.toscrape.com is a site designed for web scraping practice
    let page = browser.new_page("https://books.toscrape.com/catalogue/category/books/science_22/index.html").await?;

    // Wait for page to load
    page.wait_for_navigation().await?;

    // Extract book titles using JavaScript
    let books = page.evaluate(r#"
        Array.from(document.querySelectorAll('.product_pod h3 a'))
            .map(element => element.getAttribute('title'))
    "#).await?;

    // Convert the JavaScript value to a Rust value
    let book_titles: Vec<String> = books.into_value()?;

    Ok(book_titles)
}