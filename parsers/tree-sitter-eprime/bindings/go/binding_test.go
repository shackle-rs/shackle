package tree_sitter_eprime_test

import (
	"testing"

	tree_sitter "github.com/smacker/go-tree-sitter"
	"github.com/shackle-rs/shackle/parsers/tree-sitter-eprime"
)

func TestCanLoadGrammar(t *testing.T) {
	language := tree_sitter.NewLanguage(tree_sitter_eprime.Language())
	if language == nil {
		t.Errorf("Error loading Eprime grammar")
	}
}
