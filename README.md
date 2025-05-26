## Quickstart

Clone the repository and run:

```shell
cargo run
```

the main.rs file contains an example of a complex query defined as:
`filterText~'special characters like =<>~', age>=18, name=(jim|crossley), address='single quote address', nickname=\"use double quotes\", labels=('kubernetes.io/part-of: trustify', 'kubernetes.io/part-of: operator')"`

The output of main.rs should be the result of parsing the query:

```shell
--------------------------------------------------------------------------------------------------------------------------------------------
FIELD:'filterText'       OPERATOR:'~'    VALUE:'special characters like =<>~'
--------------------------------------------------------------------------------------------------------------------------------------------
FIELD:'age'              OPERATOR:'>='   VALUE:'18'
--------------------------------------------------------------------------------------------------------------------------------------------
FIELD:'name'             OPERATOR:'='    VALUE:'jim AND crossley'                                                        VALUE_OPERATOR:'|'
--------------------------------------------------------------------------------------------------------------------------------------------
FIELD:'address'          OPERATOR:'='    VALUE:'single quote address'                   
--------------------------------------------------------------------------------------------------------------------------------------------
FIELD:'nickname'         OPERATOR:'='    VALUE:'use double quotes'
--------------------------------------------------------------------------------------------------------------------------------------------
FIELD:'labels'           OPERATOR:'='    VALUE:'kubernetes.io/part-of: trustify AND kubernetes.io/part-of: operator'     VALUE_OPERATOR:','
--------------------------------------------------------------------------------------------------------------------------------------------
```

Notice that it contains:

- Grouping using `()` and AND(`,`) and OR(`|`)
- Special characters inside strings
- Single a double quote escaping

General ideas:

- AND operator uses `,` rather than `&` as the character `&` is a special one in URLs
- The query definition can be decoupled from the SQL and internal Models.
  - I can define a query `vulnerabilities>10` without having any column `vulnerabilities` in my internal DB tables
  - Being able to get query fields manually can allow manual translation between Client queries and internal queries.
