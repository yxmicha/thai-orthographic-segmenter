# thai-orthographic-segmenter (Python)

Python bindings for the [Thai orthographic segmenter](../README.md)
crates, as plain tokenizer functions (`word_tokenize`, `syllable_tokenize`, ...). The models are
bundled in the wheel, so there is nothing to download, and every call releases the interpreter
lock.

```sh
pip install thai-orthographic-segmenter
```

Python 3.10 or later is required.

```python
from thai_orthographic_segmenter import syllable_tokenize, word_tokenize

word_tokenize("ในปฐมกาลพระวาทะทรงดำรงอยู่")
# ['ในปฐมกาล', 'พระวาทะ', 'ทรงดำรง', 'อยู่']
syllable_tokenize("พระวาทะ")
# ['พระ', 'วา', 'ทะ']
```

