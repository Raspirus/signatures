import os
from bs4 import BeautifulSoup
import requests
import re
from modules.trienode import MD5Trie


class Virusshare:
    URL = "https://virusshare.com/hashes"
    FILES_URL = "https://virusshare.com/hashfiles"

    def __init__(self):
        self.MD5_TRIE = MD5Trie()

    def download(self):
        # Get the last file number
        last_file_nr = self.get_last_file_nr()
        # Array to store MD5 signatures
        signatures = []
        write_counter = 0

        # Loop through files from 0 to last_file_nr
        for file_number in range(last_file_nr + 1):
            # Generate the URL for the current file
            url = f'https://virusshare.com/hashfiles/VirusShare_{str(file_number).zfill(5)}.md5'
            with requests.get(url, stream=True) as response:
                for line in response.iter_lines(decode_unicode=True):
                    line = line.decode('utf-8')

                    # Extract MD5 signatures using regular expression
                    md5_signatures = re.findall(r'[a-fA-F0-9]{32}', line)

                    if len(md5_signatures) > 0:
                        if not self.MD5_TRIE.search(md5_signatures[0]):
                            # Add non-empty MD5 signatures to the array
                            signatures.extend(md5_signatures)
                            self.MD5_TRIE.insert(md5_signatures)
                            # Check the array size and write signatures to a file if it reaches 100,000
                            if len(signatures) >= 100000:
                                write_counter += 1
                                subfolder_number = (write_counter // 10)
                                # Create the "hashes" folder if it doesn't exist
                                subfolder_path = os.path.join('../hashes', str(subfolder_number).zfill(4))
                                os.makedirs(subfolder_path, exist_ok=True)
                                # Write signatures to a file
                                local_file_number = write_counter - (10 * (write_counter // 10))
                                file_path = os.path.join(subfolder_path,
                                                         f'Raspirus_{str(local_file_number).zfill(4)}.txt')
                                print(f"{write_counter}. Writing to {file_path}")
                                with open(file_path, 'a') as file:
                                    file.write('\n'.join(signatures[:100000]) + '\n')

                                # Remove the written signatures from the array
                                signatures = signatures[100000:]
                        else:
                            print("Found duplex: ", md5_signatures[0])

    def get_last_file_nr(self):
        # Send a GET request to the website
        response = requests.get(self.URL)
        html_content = response.content

        # Assuming 'html_content' contains the HTML content of the table
        soup = BeautifulSoup(html_content, 'html.parser')

        # Find all <a> tags within the table with class 'wordy'
        hash_links = soup.select('table.wordy a[href^="hashfiles/VirusShare_"]')

        # Extract the href attribute from each <a> tag and store it in a list
        hash_file_links = [link['href'] for link in hash_links]

        # Print the list of hash file links
        print(hash_file_links[-1])
        match = re.search(r'\d+', hash_file_links[-1])

        if match:
            last_digits = int(match.group())
            print(last_digits)
            return last_digits
        else:
            print("No digits found in the input string.")
            return -1

    def get_current_last_file_nr(self):
        pass
