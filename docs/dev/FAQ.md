# [Why Monorepo?](https://medium.com/@maoberlehner/monorepos-in-the-wild-33c6eb246cb9)
Cargo workspaces are a natural fit for working with a Monorepo. The main advantages of a Monorepo are:
1. When using a Monorepo approach there is one source of truth: every employee in a company or every contributor of an 
   open source project is always on the same page.
2. Large scale refactoring is very easy — changing an API which is affecting multiple parts of the codebase, can be done 
   with one commit or one pull request instead of having to touch multiple repositories for doing basically the same change 
   over and over again.
3. Collaboration across teams is easier — bugs which affect different projects, can be fixed by one person or team instead 
   of having to wait on multiple other teams to fix the same bug in their codebase. 