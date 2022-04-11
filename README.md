# wordmem-rs

`wordmem` is a helper tool for language learning, focusing on vocabulary. It takes words and explanation from user, and then makes user revisit them periodically so that user can memorize it.

Note: The syncing-by-email feature does not support IMAP ID extension and is not tested yet.

### IDEAS

The application splits to following parts:
- `word-manager`, which takes and manages words from user
- `word-visitor`, which makes user revisit the words periodically
- `revisit-planner`, which plans the revisiting schedule
- `db-syncer`, which syncs data from/to email

Revisiting means test. User need to spell out the word and the explanation respectively in 2 passes.

The revisiting is planed to start at the 1st, 2nd, 4th, 8th, 16th, 32end, 64th, 128th day since the last visiting. Correct answer will move the revisiting schedule to next planed time. On the contrary, wrong answer will move the plan backwards.

When taking words from user, user should only input a single meaning at one time, but different meanings at each time. That is, multiple meanings will be taken for the same word as time goes.

And, while doing the test, user should separate different meanings by "`;`". And user should answer all the meanings which are taken util then.

Additionally, punctuations will be normalized when comparing answers.

### DESIGN

Features:
- Storage can be synced via email.
- Security keys should be stored in system keyring.
- Words can be exported to/imported from file.

Commandline interface:
- `wordmem take`: take words from user.
- `wordmem test`: do tests.
- `wordmem signin`: sign in email to enable syncing.
- `wordmem signout`: sign out email to disable syncing.
- `wordmem push`: forcibly push data to email to keep synced.
- `wordmem pull`: forcibly pull data from email to keep synced.
- `wordmem change <word>`: change meanings of an existing word.
- `wordmem delete <word>`: delete a word.
- `wordmem open <word>`: open a word on https://translate.bing.com.
- `wordmem clear`: remove all words in DB.
- `wordmem export <file>`: export words to a file.
- `wordmem import <file>`: import words from a file.

Implementation:
- SQLite for storage of words.
- JSON format for exported file of words.
- Compressed .sqlite file as attachment of email for syncing.

License: GPL3
