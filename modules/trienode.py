class TrieNode:
    def __init__(self):
        # Each node can have 16 children (0-9 and a-f)
        self.children = {}
        # Flag to indicate the end of an MD5 hash
        self.is_end_of_hash = False


class MD5Trie:
    def __init__(self):
        # Root node of the trie
        self.root = TrieNode()

    def insert(self, md5_hash):
        node = self.root
        for char in md5_hash:
            if char not in node.children:
                node.children[char] = TrieNode()
            node = node.children[char]
        # Mark the end of the MD5 hash
        node.is_end_of_hash = True

    def search(self, md5_hash):
        node = self.root
        for char in md5_hash:
            if char not in node.children:
                # MD5 hash not found in the trie
                return False
            node = node.children[char]
        # Check if the MD5 hash ends at this node
        return node.is_end_of_hash
