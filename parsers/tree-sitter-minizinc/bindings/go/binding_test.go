package tree_sitter_minizinc_test

import (
	"testing"

	tree_sitter "github.com/smacker/go-tree-sitter"
	"github.com/shackle-rs/shackle/parsers/tree-sitter-minizinc"
)

func TestCanLoadGrammar(t *testing.T) {
	language := tree_sitter.NewLanguage(tree_sitter_minizinc.Language())
	if language == nil {
		t.Errorf("Error loading Minizinc grammar")
	}
}
