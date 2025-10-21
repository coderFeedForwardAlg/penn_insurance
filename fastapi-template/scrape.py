import os
import re
import time
import requests
from urllib.parse import urljoin, urlparse
from bs4 import BeautifulSoup

class PennScraper:
    def __init__(self, base_url):
        self.base_url = base_url
        self.visited_urls = set()
        self.session = requests.Session()
        self.session.headers.update({
            'User-Agent': 'Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/91.0.4472.124 Safari/537.36'
        })
        # Create output directory if it doesn't exist
        self.output_dir = 'scraped_pages'
        os.makedirs(self.output_dir, exist_ok=True)

    def is_valid_url(self, url):
        """Check if the URL belongs to the target domain."""
        parsed_url = urlparse(url)
        base_domain = urlparse(self.base_url).netloc
        return parsed_url.netloc == base_domain and not any(ext in url.lower() for ext in ['.pdf', '.jpg', '.jpeg', '.png', '.gif'])

    def get_filename_from_url(self, url):
        """Generate a filename based on the URL."""
        parsed = urlparse(url)
        path = parsed.path.strip('/')
        if not path:
            return 'penn.txt'
        
        # Remove any query parameters
        path = path.split('?')[0]
        # Replace slashes with underscores and remove any remaining special characters
        filename = re.sub(r'[^\w\-]', '_', path)
        # Remove consecutive underscores and trailing/leading ones
        filename = re.sub(r'_+', '_', filename).strip('_')
        return f"penn_{filename}.txt" if filename else 'penn.txt'

    def extract_text(self, soup):
        """Extract and clean text from the BeautifulSoup object."""
        # Remove script and style elements
        for script in soup(["script", "style", "nav", "header", "footer"]):
            script.decompose()
        
        # Get text and clean it up
        text = soup.get_text()
        lines = (line.strip() for line in text.splitlines())
        chunks = (phrase.strip() for line in lines for phrase in line.split("  "))
        text = '\n'.join(chunk for chunk in chunks if chunk)
        return text

    def save_content(self, url, content):
        """Save the scraped content to a file."""
        filename = os.path.join(self.output_dir, self.get_filename_from_url(url))
        with open(filename, 'w', encoding='utf-8') as f:
            f.write(content)
        print(f"Saved: {filename}")

    def get_links(self, url, soup):
        """Extract all valid internal links from the page."""
        links = set()
        for link in soup.find_all('a', href=True):
            href = link['href']
            # Join relative URLs with base URL
            full_url = urljoin(url, href)
            # Remove fragments
            full_url = full_url.split('#')[0]
            if self.is_valid_url(full_url) and full_url not in self.visited_urls:
                links.add(full_url)
        return links

    def scrape_page(self, url):
        """Scrape a single page and return its links."""
        if url in self.visited_urls:
            return set()
            
        print(f"Scraping: {url}")
        self.visited_urls.add(url)
        
        try:
            response = self.session.get(url, timeout=10)
            response.raise_for_status()
            
            soup = BeautifulSoup(response.text, 'html.parser')
            
            # Extract and save text content
            text_content = self.extract_text(soup)
            self.save_content(url, text_content)
            
            # Be polite and don't overload the server
            time.sleep(1)
            
            # Return all valid links from this page
            return self.get_links(url, soup)
            
        except requests.RequestException as e:
            print(f"Error scraping {url}: {e}")
            return set()

    def scrape_site(self, start_url=None):
        """Start scraping the website from the given URL."""
        if start_url is None:
            start_url = self.base_url
            
        to_visit = {start_url}
        
        while to_visit:
            current_url = to_visit.pop()
            new_links = self.scrape_page(current_url)
            to_visit.update(new_links - self.visited_urls)

def main():
    base_url = 'https://www.pennnationalinsurance.com/'
    scraper = PennScraper(base_url)
    print(f"Starting to scrape {base_url}")
    scraper.scrape_site()
    print("Scraping completed!")

if __name__ == "__main__":
    main()