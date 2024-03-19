package tree_sitter_datazinc_test

import (
	"testing"

	tree_sitter "github.com/smacker/go-tree-sitter"
	"github.com/shackle-rs/shackle/tree-sitter-datazinc"
)

func TestCanLoadGrammar(t *testing.T) {
	language := tree_sitter.NewLanguage(tree_sitter_datazinc.Language())
	if language == nil {
		t.Errorf("Error loading Datazinc grammar")
	}
}
