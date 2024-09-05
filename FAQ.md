Frequently Asked Questions
===

What is Afrim?
---

Afrim is an input method engine (IME) to type any languagues that can be represented using a sequential code.

Afrim is also a library/tool to implement IME.

**Disclaimer**: Afrim is not a writing system. Instead, it uses existing writing systems to operate.

What means Afrim?
---

"afrim" who is the transliteration of the Amharic word "እፍሪም," meaning shameful.

"Shameful" symbolizes embracing vulnerability and self-awareness, turning mistakes into opportunities for growth and transformation.

**Note**: Whether you use "afrim" or "Afrim," it doesn't matter. 

How Afrim works?
---

Using the transliteration, Afrim transforms what you type (in English character) to your native language.

- Eg. Afrim -> እፍሪም (Amharic input method)
- Eg. Pookai2t peu2nze22 n*kut -> Pookɛ́t pә́nzéé ŋkut (Bamun input method)

Technically, each character of the target language can be represented by a sequential code (sequence of characters).

**Example:**
```
A -> እ
f -> ፍ
ri -> ሪ
m -> ም
```

More details about the internal working can be found at [afrim-memory](memory/src/lib.rs).


Why a phonetic input method?
---

[Geʽez] and [clafrica] was the first implemented input methods in Afrim.
Both are based on the phonetic representation of what you want to type and the design of the Afrim has been influenced by them.

**Advantages**

- **Ease of use**: For people familiar with the Latin alphabet, the phonetic input method is more intuitive and accessible than memorizing the language specific keyboard layout.

- **Efficiency**: It allows for faster typing because users can rely on the phonetic sounds they are familiar with, rather than hunting for specific characters on a specialized keyboard.

**Note**: If your language don't offer this writing system, feel free to propose your own to the community.
 Confer [afrim-data].

Which problems Afrim wants to solve?
---

Current IMEs, allows users to type in their own dialects. But, be able to type, is not the only challenge that a person can encountered.

- Typo

For a person who is still learning the language, it's easy to make mistakes.
 Eg: `አፍሪም (afrim)` means `Shameful` while `ዐፍሪም (Afrim)` means `Aphraim`.

- Stress

Imagine that you have to type this number `99` in nufi language ` ncɔ̀ vʉ̀'ʉ̄ mɑ̀ mʉ̄vʉ̀'ʉ̄'`.
 Or type this number `1234567890` in geez `፲፪፼፴፬፻፶፮፼፸፰፻፺`. The easiest way is to use the hard-coded format.

- Memoryless

For a person who don't master the language, it's easy to forget how to type a particular word.

What Afrim brings?
---

**Autosuggestion**

Afrim assists you while you are typing.

**Autocorrection**

Afrim permits you to correct error as you type.

**Date and number translator**

Do you want to type `፼፳፫፻፵፭፼፷፯፻፹፱፼፻፳፫` in geez? Just type `1234567890123` and Afrim will suggests the equivalents in geez.

Why not propose a solution that is language specific?
---

We can have input method specific for each language, but at the end, we will be reinventing the road.
 Think about the similarity between writing systems (not the alphabet but the working principle).

Afrim doesn't aim to be the alone solution, but seeks to make accessible the IME technology to everyone.

To accomplish it, Afrim has been designed to be open source, modular, well documented (source code) and not language specific.

You can use Afrim as a library or tool to implement your own IME.
 Confer [afrim-memory], [afrim-preprocessor], [afrim-translator], [afrim-data].

How does this compare to IBus or Fcitx?
---

Afrim can be used as a backend for IBus and Fcitx.

Why solve the problem in one application all together?
---

Afrim is not just an IME, it's a tool who can be used to implement an IME.

By example, if you want Afrim acts as a geez IME or swahili IME or ewondo IME, ..., just provide to it the configutration file that suit your need.

Is this being done in collaboration with the linguistics faculties of African universities?
---

No, Afrim is a tool and not a writing system. Its datasets are provided by the community and/or official sources.
 And we will make sure that the original author of the dataset is credited.

[afrim-memory]: memory/
[afrim-preprocessor]: engine/preprocessor/
[afrim-translator]: engine/translator/
[afrim-data]: https://github.com/pythonbrad/afrim-data/
